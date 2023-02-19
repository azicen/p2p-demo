use std::net::{UdpSocket, SocketAddr};
use std::io;
use std::str;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {
    let addr = "120.24.87.100:25565";
    let client = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(err) => panic!("启动失败: {}", err)
    };
    println!("客户端尝试在[{}]上建立Socket", client.local_addr().unwrap());
    println!("尝试连接会合服务器: [{}]", addr);
    client.connect(addr).unwrap();
    client.send("hi".as_bytes()).unwrap();

    println!("等待其他客户端接入");
    let mut buf = [0; 1024];
    let number_of_bytes = match client.recv(&mut buf) {
        Ok(size) => size,
        Err(err) => {
            panic!("接收数据时发生错误: {}", err);
        }
    };
    let mut user:Option<SocketAddr> = Default::default();
    let filled_buf = &mut buf[..number_of_bytes];
    match str::from_utf8(&filled_buf) {
        Ok(message) => {
            println!("获取到对方客户端信息: [{}]", message);
            user = Some(message.parse().unwrap());
        },
        Err(err) => {
            panic!("接受到来自服务器的错误消息: {}, 消息原文: {:?}", err, filled_buf);
        }
    }

    // 向对方客户端发送任意消息，此消息将被对方nat丢弃（非法来源）
    // 作用是在我方nat打开一个可进入的通道，使得对方可以发送消息而不被我方nat丢弃
    client.connect(user.unwrap()).unwrap();
    let _ = client.send("open".as_bytes());
    // 延迟3秒，等待对方打开nat
    thread::sleep(Duration::from_secs(1));
    if let Err(err) = client.send(format!("here it is {}", client.local_addr().unwrap()).as_bytes()) {
        panic!("与客户端[{}]创建UDP失败: {}", user.unwrap(), err);
    }

    client.set_nonblocking(true).unwrap();

    let (tx, rx):(Sender<String>, Receiver<String>) = channel();

    thread::spawn(move || {
        loop {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            tx.send(buf).unwrap();
        }
    });

    loop {
        let mut buf = [0; 1024];
        let (number_of_bytes, src_addr) = match client.recv_from(&mut buf) {
            Ok(res) => res,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                (usize::try_from(0).unwrap(), "0.0.0.0:0".parse().unwrap())
            },
            Err(err) => {
                panic!("接收数据时发生错误: {}", err);
            }
        };
        if number_of_bytes > 0 {
            let filled_buf = &mut buf[..number_of_bytes];
            println!("接收到来自[{}]的消息 > {}", src_addr, str::from_utf8(&filled_buf).unwrap().trim_end());
        }

        match rx.try_recv() {
            Ok(message) => {
                if !message.is_empty() {
                    if let Err(err) = client.send(message.as_bytes()) {
                        panic!("发送信息失败：{}", err);
                    }
                }
            },
            Err(err) => {
                if let TryRecvError::Disconnected = err {
                    panic!("输入管道发生错误：{}", err);
                }
            }
        }

        thread::sleep(Duration::from_nanos(50));
    }
}
