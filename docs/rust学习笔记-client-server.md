# rust学习笔记-client/server networking

> 这周才写完project3，下周要加快进度了，下周五之前要把project45都写完
>
> 最近好听的歌太多啦，前两天每天循环"Love me like that"，这两天一写代码脑子里就开始放"Teddy Bear~ wuwuwuwu~"

[toc]

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

```rust
pub fn new(addr: SocketAddr, engine: E) -> Result<KvServer<E>> {
    let listener = TcpListener::bind(addr)?;
    info!("bind to {}", addr);
    Ok(KvServer { addr, listener, kvengine: engine })
}
```

```rust
pub fn new(addr: SocketAddr) -> Result<KvClient> {
    let mut stream = TcpStream::connect(addr)?;
    let writer = BufWriter::new(stream.try_clone()?);
    let reader = BufReader::new(stream);
    info!("connected");
    Ok(KvClient { addr, writer, reader })
}
```



最开始使用了简单的stream.write()和stream.read_to_string()来进行消息的接收和发送，但是write的时候不会写入一个EOF，但是read_to_string()需要读到EOF才停止，因此，会卡在read的时候，感觉比较合理的解决办法是先写入长度，再写入数据，读的时候就可以读取指定长度，serde_json的Deserializer在内部实现了这种机制，因此采用了Deserializer::from_reader().into_iter::\<Request>()的方法（这好象叫粘包问题，解决方法就是加报头）。

```rust
for stream in self.listener.incoming() {
    let mut stream = stream.unwrap();
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let request_iter = Deserializer::from_reader(reader).into_iter::<Request>();
    for request in request_iter {
        ...
    }
    ...
}
```

### trait的使用

由于项目中需要两种engine：自己实现的kvstore和sled（一个嵌入式数据库），因此需要定义一个trait（类似于C++的接口）KvEngine，定义三个共同方法set、get、remove，KvStore和KvSledStore都需要实现这三个方法，这里比较特殊的语法就是如果想要使用trait定义的方法，那么需要引入该triat，比如use crate::KvEngine。在KvServer中需要定义一个实现了该trait的成员变量kvengine，需要在struct定义的地方指明：

```rust
pub struct KvServer<E: KvEngine> {
    addr: SocketAddr,
    listener: TcpListener,
    kvengine: E,
}
```

在impl的时候也要指明使用的trait：

```rust
impl<E: KvEngine> KvServer<E> {
    /// construct a new server
    pub fn new(addr: SocketAddr, engine: E) -> Result<KvServer<E>> {
        let listener = TcpListener::bind(addr)?;
        info!("bind to {}", addr);
        Ok(KvServer { addr, listener, kvengine: engine })
    }
    ...
}
```

如果想要在某个函数中使用KvEngine，那么在函数声明处也要标名trait，main函数是不可以指定trait的，因此在创建server的时候需要抽象一个函数来创建server：

```rust
fn main() -> Result<()> {
    ...
    let path = current_dir()?;
    match engine_type {
        Engine::kvs => {
            run_server(KvStore::open(path)?, opt.addr)?;
        },
        Engine::sled => {
            run_server(KvSledStore::open(path)?, opt.addr)?;
        },
    }
    Ok(())
}

fn run_server<E: KvEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    info!("run server");
    let mut server = KvServer::new(addr, engine)?;
    server.run()?;
    Ok(())
}
```

### sled

sled是一个嵌入式数据库，类似于一个线程安全的BTreeMap<[u8], [u8]>，调用insert、get、remove接口就可以，这里可能出现的bug是由于测试文件会kill掉server进程，如果在kill的时候数据还没有持久化到磁盘里，那么下一次recover的时候就无法恢复到原来的期望的状态，因此需要在每次insert、remove之后都调用flush，否则可能数据还没有持久化到磁盘中server就被kill掉了。

