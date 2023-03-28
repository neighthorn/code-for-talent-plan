#  rust学习笔记-kvstore project1&2

> 回到了刚开始学编程的日子
>
> 某个人让我写学习笔记，但是既不follow也不star，不知道什么时候才会发现这条吐槽

[toc]

## KvStore-description

KvStore共五个project，目前完成了project1&2，前两个project主要是完成一个args_parser和一个in-memory kv-store。

- project1：https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md

- project2：https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-2/README.md

## rust小知识

### clap

clap主要用于命令行参数解析和生成帮助说明（也就是--help）。

- clap会自动提供--help和--version参数，可以通过version()、author()、about()方法来提供程序的一般信息；
- 通过Arg::with_name()可以创建一个命名参数，.help()方法可以提供该参数的说明信息；
- .subcommand()方法可以传入SubCommand对象，构造子命令

使用示例如下：

```rust
let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .arg(Arg::with_name("-V").help("print the version of package"))
    .subcommand(
        SubCommand::with_name("get")
            .about("get value for the given key")
            .arg(Arg::with_name("KEY").help("A string key").required(true)),
    )
    .subcommand(
        SubCommand::with_name("set")
            .about("set value for the given key")
            .args(&[
                Arg::with_name("KEY").help("A string key").required(true),
                Arg::with_name("VALUE")
                    .help("A string value")
                    .required(true),
            ]),
    )
    .subcommand(
        SubCommand::with_name("rm")
            .about("remove the key-value pair for the given key")
            .arg(Arg::with_name("KEY").help("A string key").required(true)),
    )
    .get_matches();

match matches.subcommand() {
    ("set", Some(args)) => {
        ...
    }
    ("get", Some(args)) => {
        ...
    }
    ("rm", Some(args)) => {
        ...
    }
    _ => {
        exit(1);
    }
}
```

### serde

serde用于对rust数据结构进行序列化和反序列化，project2中使用serde来对Command进行序列化和反序列化。

首先需要在Cargo.toml中添加两个dependencies：

```
[dependencies]
serde = "1.0.158"
serde_json = "1.0"
```

对于需要序列化和反序列化的数据结构，需要指定#[derive(Serialize, Deserialize)]属性：

```rust
/// 'Command' is a enum that represents various commands
#[derive(Serialize, Deserialize)]
pub enum Command {
    Set{key: String, value: String},
    Rm{key: String},
}
```

在project2中，需要将Command命令写入到文件中。首先需要把Command对象序列化成string，然后使用BufWriter来进行写入：

```rust
// construct set command
let cmd = Command::Set{ key, value };
// serialize the command to a string
let write_str = serde_json::to_string_pretty(&cmd)?;
// write command into the disk
let len = self.writer.write(write_str.as_bytes())?;
self.writer.flush()?;
```

project2中有两个地方需要对Command进行反序列化读取，一个是读取指定位置的Command对象，一个是把整个文件反序列化成Command迭代器。

读取指定位置的Command对象时，首先通过BufReader把数据读入缓冲区，需要通过.take()方法来指定读取的字节长度，然后使用from_reader()来进行反序列化：

```rust
let mut file = File::open(self.path.clone())?;
file.seek(SeekFrom::Start(*pos))?;
let reader = BufReader::new(file).take(*len);
// read command from the log file
if let Command::Set { value, .. } = serde_json::from_reader(reader)? {
    // return the value
    return Ok(Some(value));
} else {
    return Err(KvStoreError::GetNonExistValue);
}
```

对于一整个文件的反序列化，可以使用Deserializer::from_reader().into_iter::<Command>()来生成迭代器，使用next方法来进行迭代遍历：

```rust
let mut stream = Deserializer::from_reader(file).into_iter::<Command>();
let mut pos = 0;
while let Some(cmd) = stream.next() {
    let new_pos = stream.byte_offset() as u64;
    match cmd? {
        Command::Set{key, ..} => {
            index.insert(key, (pos, new_pos - pos));
        }
        Command::Rm { key } => {
            index.remove(&key);
        }
    }
    pos = new_pos;
}
```

### 自定义错误类型

自定义错误主要使用了[failure crate](https://boats.gitlab.io/failure/guidance.html)，在project2中使用了[A Custom Fail type](https://boats.gitlab.io/failure/custom-fail.html#caveats-on-this-pattern)来定义了KvStoreError，可以通过KvStoreError来处理所有的错误类型（包括rust提供的错误类型），对于自定义的错误，只需要实现fail trait，对于其他crate提供的错误需要添加#[cause]来执行错误类型来源：

```rust
/// Error type for KvStore
#[derive(Fail, Debug)]
pub enum KvStoreError {
    /// try to get the value of a non-existent key
    #[fail(display = "get value for the non-existent key")]
    GetNonExistValue,
    /// try to remove a non-existent key
    #[fail(display = "remove non-existent key")]
    RemoveNonExistKey,
    /// fail to serialize a command to the log file
    #[fail(display = "fail to serialize a command to the log file")]
    SerializeCmdError,
    /// fail to rebuild the in-memory index
    #[fail(display = "fail to rebuild the in-memory index")]
    RebuildIndexError,
    /// io error
    #[fail(display = "io error: {}", _0)]
    Io(#[cause] io::Error),
    /// serde error
    #[fail(display = "serde error: {}", _0)]
    Serde(#[cause] serde_json::Error),
}
impl From<io::Error> for KvStoreError {
    fn from(err: io::Error) -> KvStoreError {
        KvStoreError::Io(err)
    }
}
impl From<serde_json::Error> for KvStoreError {
    fn from(err: serde_json::Error) -> KvStoreError {
        KvStoreError::Serde(err)
    }
}
```

如GetNonExistValue就是一个自定义的错误类型，只实现了fail trait，在KvStoreError中定义Io来处理io::Error，因此需要添加#[cause]来指定错误类型来源。

通过定义Result type来让KvStoreError处理系统中的所有错误：

```rust
pub type Result<T> = std::result::Result<T, KvStoreError>;
```

**一些小bug**：在实现错误处理的时候遇到了一些小bug，由于一开始在kvstore.rs中引入了std::io::Result，因此在kvstore.rs中的所有Result都默认使用了std::io::Result，而没有使用上述自定义的Result type，因此会报错。

### 文件操作

正常文件的创建打开可以使用File::create()和File::open()，但是create的时候，如果文件已经存在会清空原有内容，如果文件不存在会创建文件，并且create后获得的文件句柄是一个只写的模式，如果去读文件就会抛出Bad file descriptor的错误，因此可以使用OpenOptions::new().read(true).write(true).create(true).open()来创建文件并以读写模式打开。

```rust
let mut file = if path.clone().as_path().exists() {
    OpenOptions::new().read(true).write(true).open(path.clone())?
} else {
    OpenOptions::new().read(true).write(true).create(true).open(path.clone())?
};
```



