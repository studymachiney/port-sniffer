use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Sender};
use bpaf::Bpaf;
use tokio::net::TcpStream;
use tokio::task;


/// 最大 IP 端口
const MAX: u16 = 65535;

/// 默认 IP 地址
const IPFALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Arguments {
    // 地址参数 接收 -a 和 --address
    #[bpaf(long, short, argument("Address"), fallback(IPFALLBACK))]
    /// 需要嗅探的地址必须是有效的 ipv4 地址
    pub address: IpAddr,
    #[bpaf(
        long("start"),
        short('s'),
        guard(start_port_guard, "必须大于0"),
        fallback(1u16)
    )]
    /// 嗅探起始端口（必须大于0）
    pub start_port: u16,
    #[bpaf(
        long("end"),
        short('e'),
        guard(end_port_guard, "需小于等于65535"),
        fallback(MAX)
    )]
    /// 嗅探截止端口（需小于等于65535）
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX
}

// 扫描端口
async fn scan(tx: Sender<u16>, port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).unwrap();
        },
        Err(_) => {}
    }
}

#[tokio::main]
async fn main() {
    let opts = arguments().run();

    let (tx, rx) = mpsc::channel();
    for port in opts.start_port..opts.end_port {
        let tx = tx.clone();
        
        task::spawn(async move {
            scan(tx, port, opts.address).await;
        });
    }

    drop(tx);
    let open_ports: Vec<u16> = rx.iter().collect();

    if open_ports.is_empty() {
        println!("没有开放的端口在 {}", opts.address);
    } else {
        println!("\n{} 条开放的端口在 {}", open_ports.len(), opts.address);
        open_ports.iter().for_each(|p| println!("端口 {} 开放", p));
    }
}
