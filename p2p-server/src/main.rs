use std::net::{UdpSocket, SocketAddr};
use std::str;

fn main() {
    let addr = "0.0.0.0:25565";
    println!("会合服务器尝试监听: [{}]", addr);
    let socket = match UdpSocket::bind(addr) {
        Ok(socket) => {
            println!("服务器监听成功");
            socket
        },
        Err(err) => panic!("监听失败: {}", err)
    };
    println!("");

    let mut waiting_user:Option<SocketAddr> = Default::default();
    loop {
        let mut buf = [0; 1024];
        let (number_of_bytes, src_addr) = match socket.recv_from(&mut buf) {
            Ok(res) => res,
            Err(err) => {
                println!("接收数据时发生错误: {}", err);
                continue;
            }
        };

        let filled_buf = &mut buf[..number_of_bytes];
        match str::from_utf8(&filled_buf) {
            Ok(message) => {
                println!("接受到来自[{}]的消息 > {}", src_addr, message)
            },
            Err(err) => {
                println!("接受到来自[{}]的错误消息: {}, 消息原文: {:?}", src_addr, err, filled_buf);
                continue;
            }
        }

        let user1 = src_addr;
        let user2 = match waiting_user {
            Some(user) => user,
            None => {
                waiting_user = Some(user1);
                println!("用户[{}]进入等待队列", user1);
                continue;
            }
        };

        println!("开始建立[{}]--[{}]的连接", user1, user2);
        waiting_user = None;
        if let Err(err) = socket.send_to(user1.to_string().as_bytes(), user2) {
            println!("会合连接建立失败: {}", err);
            continue;
        }
        if let Err(err) = socket.send_to(user2.to_string().as_bytes(), user1) {
            println!("会合连接建立失败: {}", err);
            continue;
        }
        println!("连接[{}]--[{}]建立成功", user1, user2);
        println!("");
    }

}
