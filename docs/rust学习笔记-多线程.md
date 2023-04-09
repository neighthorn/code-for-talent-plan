# rust学习笔记-多线程

> 感觉多线程的使用需要对rust中的所有权有一定的理解
>
> 这周跳舞发现脑子和体能都跟不上了，学习效率高一点，多练练体能
>
> 感冒了，一周没办法跳舞呜呜，感冒耽误学习还耽误跳舞
>
> 终于在下周开始之前写完了这个course，虽然感觉从project2开始我的代码就像一坨懒羊羊的头发，写到project4只不过是在懒羊羊的头发上继续复刻一坨头发，顺便捏捏把两坨头发~~完美~~融合在一起

## 多线程kvstore

project4需要把kv-server改成多线程server，使用线程池来处理client的request。

在把原来的kvstore改成concurrent_kvstore的过程中，一开始保留了原来的BTreeMap来维护in-memory index，但是rust内部的BTreeMap本身不是线程安全的，因此需要用Mutex来保证并发的正确性，但是这样如果想要读也要去获取独占锁，实际上并没有做到真正的并发，所以参考example的代码把BTreeMap换成了SkipMap，这样在读的时候不需要手动去加互斥锁。对于writer还是需要使用互斥锁来进行正确性的保证，但是由于每次读的时候都是创建一个新的reader，因此reader也不需要加锁，这样，如果在线程池中线程足够多的情况下，就可以保证读不被阻塞。

## rust小知识

### 闭包

这个虽然没有在project4中直接用到，但是在理解多线程中资源竞争时对闭包也进行了一定的学习，主要是Fn、FnMut、FnOnce等trait的理解。

https://zhuanlan.zhihu.com/p/341815515

### Arc和Mutex

https://juejin.cn/post/7104936990788812807#heading-3

这大概是这个project里面唯一一个会用的东西，虽然原理理解得一塌糊涂。

首先，rust中的所有权遵循以下规则：

- 同一时刻一个数据只有一个所有者；
- 如果变量x的数据存储在栈上，let y=x; 会把数据复制给y，如果变量x的数据存储在堆上，那么let y=x; 会把数据的所有权转移给y，x不再拥有数据的所有权，也就不再可使用；
- 在同一时刻只允许存在一个可变引用（避免了数据竞争）；
- 同一时刻可以存在多个不可变引用；
- 不能够在拥有不可变引用的同时拥有可变引用；

为了能够在多线程中使用共享数据，需要用到Arc和Mutex。Arc（Atomic Reference Counter）是线程安全的智能指针，采用引用计数的方法，追踪所有指针的拷贝，当所有指针拷贝都离开作用域之后才会对数据进行释放；Mutex是一个互斥锁，通过.lock().unwrap()来获取数据的使用权，通过drop()可以显式地释放锁，没有unlock方法。

### 线程池

线程池的创建主要涉及到了线程间的通信，其中sharedQueueThreadPool模仿了Rust中文书中的多线程web server，主要通过信道来进行线程间的通信（这里主要是把任务发送给线程），mpsc是一个多生产者，单消费者的实现，通过创建receiver的Arc指针，可以让多个worker来共享receiver，就可以实现“多消费者”。

https://kaisery.github.io/trpl-zh-cn/ch20-02-multithreaded.html

### catch_unwind

其实没太理解AssertUnwindSafe的用法，catch_unwind主要用来捕获panic，我们希望一个thread发生panic之后还能够对该线程进行复用，因此需要对panic进行捕获，有点类似C++里面的try_catch的用法。
