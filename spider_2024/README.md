# 爬虫 24 重构版本

## 1. 概览

### 1.1 技术栈

| 名称    | 文档地址                                    | 备注                                                                                       |
| ------- | ------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Redis   | <https://redis.io/docs/latest/>             | 缓存中间件。爬虫目前使用 Redis 来缓存个人门户、大物实验平台等的密码                        |
| Salvo   | <https://salvo.rs/zh-hans/guide/index.html> | 一个 HTTP 框架。                                                                           |
| Sqlx    | <https://docs.rs/sqlx/latest/sqlx/>         | 操作数据库（如 MySQL）的框架。爬虫目前主要在获取个人门户、大物实验平台的密码时用到数据库。 |
| Tokio   | <https://docs.rs/tokio/latest/tokio/>       | 异步运行时                                                                                 |
| Reqwest | <https://docs.rs/reqwest/latest/reqwest/>   | 用来发 HTTP 请求的框架                                                                     |
| Moka    | <https://docs.rs/moka/latest/moka/>         | 一个缓存库，和 Redis 不同，他是直接缓存在爬虫内存中的。主要用来缓存湖大各种平台的登陆状态  |

### 1.2 项目结构

```text
|-- .gitea          // gitea 的 action 配置
|-- Cargo.toml      // 项目依赖配置
|-- logs            // 日志文件
|-- rustfmt.toml    // rust代码格式化配置
|-- src             // 源代码
`-- target          // 编译文件
```

```text
|-- app_error.rs    // 自定义的错误类型
|-- app_result.rs   // 自定义的结果类型
|-- config.rs       // 配置文件的定义
|-- main.rs         // 入口文件
|-- middlerwares.rs // 各种中间件
|-- router.rs       // 定义路由
|-- handlers        // 存放各种结构体
|   `-- ...
|-- spiders
|   |-- login.rs    // 这个文件集成了各种平台的登录函数
|   `-- ...         // 存放具体的爬虫逻辑
|-- handlers        // 存放各种路由
|   `-- ...
`-- utils
    |-- cache.rs    // 使用 moka 缓存登陆状态
    |-- captcha.rs  // 验证码识别服务，主要用于解决大物实验平台的登录验证码
    |-- crypto.rs   // 用户的密码在数据库中都是加密存放的，这里提供了加解密的函数，同时也包括请求个人门户和大物实验平台登录请求需要的加密函数
    |-- db.rs       // 数据库相关操作，主要用于获取密码
    |-- redis.rs    // redis 相关操作，主要用于缓存获取的密码
    |-- request.rs  // 发送 http 请求
    `-- traffic.rs  // 一个中间件，用于统计爬虫的负载情况
```

## 2. 快速开始

爬虫的配置应由调用方（通常为后端）传递。可查看 [config.rs](./src/config.rs) 确认有哪些配置项。

目前只有大物实验平台需要用到验证码解析服务，如果你不需要调试大物实验平台相关代码，那么 `captcha_url` 无需配置。否则，你需要根据 captcha_service 配置验证码解析服务。

建议将 `config/config.toml` 中的 `filter_level` 设为 `debug`，`with_ansi` 设为 `true`，`to_stdout` 设为 `true`，以方便调试。

## 3. 开发

### 3.1 请求过程

当后端向爬虫请求数据时，将根据 `router.rs` 中关于路由的定义，将请求交给 `handlers` 内的函数处理。`handlers` 内的函数一般会调用 `spiders` 内的函数来获取输出并返回给后端。

假如说我们现在要获取某个学号在校园网系统中的某个接口的数据，那么该请求最终会交给 `spiders/netflow.rs` 内的某个具体的函数进行处理。这个函数一般会首先获取当前学号的登录状态，即我们会看到 `let netflow_headers = netflow_headers(stu_id).await?;` 这样的代码。这个代码会去调用 `spiders/login.rs` 中的 `netflow_headers` 函数。而 `netflow_headers` 函数首先会从 Moka 中看一下之前是否已经缓存了登录状态（缓存配置相关代码在 `utils/cache.rs` 中）。如果已经缓存了则会立刻返回，否则的话，`netflow_headers` 将会使用用户的学号和密码来向校园网系统发送登录请求，并把登陆状态（一般是一些请求头，主要的就是 Cookie）缓存下来。`spiders` 内的函数拿到了登录状态就可以继续向校园网系统发送请求。

湖大目前各个系统的鉴权过程大概是，在 CAS 统一身份认证平台，通过个人门户的账号密码完成认证，认证通过后将会下发一个叫 `ticket` 的东西，然后我们再拿着 `ticket` 去请求不同的系统就会完成鉴权。本项目 `spiders/login.rs` 中的 `get_ticket_url` 就完成了请求 CAS ，自动填写账号密码并拿到 `ticket` 的过程。具体的过程可以参考代码或是自行抓包。湖大不同的系统的鉴权方式也有所区别。

### 3.2 调试

如果你考虑使用 HTTP 接口测试工具来进行调试，那么你需要在请求时，增加 `Authorization` 请求头，内容为 `OUJhbGciOiJIUzU(x7)iIsImlhdCI6MTYxNzQy$jAwMiwiZXh#IjoxNjUzNDI2MDAyfQ@eyI6ImFkbWhjXzxcwEiT7dlm9sFeSRlgY7rnJKpBA`（参考 `middlewares.rs`）

你还可以运行单元测试来进行调试。`spiders` 中的各个函数基本都在代码文件的最下方有对应的单元测试，如：

```rust
#[tokio::test]
async fn test_get_netflow() {
    let res = get_netflow(&STU_ID).await.unwrap();
    dbg!(res);
}
```

你的 IDE 应该能自动识别出这些单元测试函数，然后提供直接执行该函数的运行按钮。

为了方便起见，我们在 `request.rs` 中定义了一个全局变量 `STU_ID`，所有的单元测试都会使用这个学号。你可以将这个变量改为自己的学号，但是需要确保该学号的信息在你连接的数据库的 `mini_bind` 表中存在（或者说你可能需要实现已经从后端或是前端添加了关于该学号的绑定）。

当然，如果你增加了 `spiders` 内的函数，那么你也需要像别的函数一样为其写一个单元测试。

### 3.3 爬虫与后端

我们按如下的原则划分爬虫和后端的职责。当然现在的代码由于历史原因可能未必全部符合，如果你发现了不符合的代码，你可以纠正并贡献代码。

- 爬虫应像一个代理一样为后端进行数据爬取。
- 爬虫应尽可能返回爬取到的原始数据，数据的解析工作应交给后端。
- 有关鉴权，登录状态失效等问题的处理应该交给爬虫，后端只希望调用爬虫后可以直接拿到想要的数据。
- 如果需要当前用户的信息（如当前这个学号是哪个年级，什么性别等），需要由后端获取，并携带在与爬虫的请求中。爬虫只需要关心爬取数据。
