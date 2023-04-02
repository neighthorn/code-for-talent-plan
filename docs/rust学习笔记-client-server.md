# rust学习笔记-client/server networking

> 这周能不能写完project3
>
> 最近好听的歌太多啦，前两天每天循环"Love me like that"，这两天一写代码脑子里就开始放"Teddy Bear~ wuwuwuwu~"

## Project3 client-server networking

project3主要把原来的命令行kvstore封装成client-server的形式

## rust小知识

### StructOpt

StructOpt可以将命令行参数解析成一个结构体，可以更加方便地指定参数格式。

在client和server接收的参数中，addr需要是ipv4或者ipv6的格式，以client的参数解析为例，可以通过subcommand的形式来定义set、get、rm参数，其中addr需要是SocketAddr的格式（也就是ip:port的格式），可以通过default_value来指定默认值：

```rust
#[derive(Debug, StructOpt)]
struct Set {
    key: String,
    value: String,
    #[structopt(long="addr", default_value="127.0.0.1:4000")]
    addr: SocketAddr,
}
...
#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "get")]
    Get(Get),
    #[structopt(name = "set")]
    Set(Set),
    #[structopt(name = "rm")]
    Rm(Rm),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client")]
pub struct Arguments {
    #[structopt(subcommand)]
    command: Command,
}
```

如果某个参数的值希望是enum，可以用arg_enum!宏来对enum进行配置，比如kvs-server中的Engine：

```rust
/// engine type, kvs or sled
arg_enum! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Engine {
        kvs,
        sled,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-server")]
struct Arguments {
    #[structopt(long, default_value="127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long)]
    engine: Option<Engine>,
}
```

arg_enum!自动为enum实现了FromStr、Display trait，默认case_insensitive

StructOpt实际上是把这些结构体转换成了clap配置，对于参数格式的限制也可以通过.validator()来完成，但是编写clap会比较复杂。

参考资料：

https://github.com/TeXitoi/structopt/tree/master/examples

https://colobu.com/2019/09/29/rust-lib-per-week-structopt/

### 日志：log和env_logger

log提供了日志记录接口，通过info!()、debug!()等宏来记录消息，env_logger配合log来使用，默认将日志输出到标准错误流(stderr)，可以通过target()来指定输出位置，可以通过.filter_level()来进行日志过滤，比如kvs-server中将filter_level设置为了Info，则debug和trace都不会被输出。

### c/s通信：TcpListener和TcpStream

Server使用TcpListener::bind()来绑定到一个ip-port，监听connection，通过TcpStream来进行消息的收发；Client使用TcpStream::connect()来连接到一个ip-port，通过TcpStream来进行消息的收发。

