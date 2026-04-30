---
title: 默认模块
language_tabs:
  - shell: Shell
  - http: HTTP
  - javascript: JavaScript
  - ruby: Ruby
  - python: Python
  - php: PHP
  - java: Java
  - go: Go
toc_footers: []
includes: []
search: true
code_clipboard: true
highlight_theme: darkula
headingLevel: 2
generator: "@tarslib/widdershins v4.0.30"

---

# 默认模块

Base URLs:

# Authentication

# Default

## POST 登录

POST /user/login

使用个人门户账号登录，后端将调用微生活的数据库进行鉴权。

可能的比较特殊的错误情况：

* PERMISSION_DENIED：表示该账号被禁止使用试卷库
* NOT_BIND_WEIHUDA：该账号没有绑定湖大微生活
* PASSWORD_ERROR：密码错误

> Body 请求参数

```yaml
stuid: ""
password: ""

```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» stuid|body|string| 否 |学号|
|» password|body|string| 否 |个人门户密码|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": {
    "token": "string",
    "user": {
      "stu_id": "string",
      "name": "string",
      "permissions": [
        "string"
      ]
    }
  },
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|object|false|none|token|none|
|»» token|string|true|none||none|
|»» user|[User](#schemauser)|true|none||none|
|»»» stu_id|string|true|none|学号|none|
|»»» name|string|true|none|姓名|none|
|»»» permissions|[string]|true|none|权限|none|
|» msg|string¦null|false|none|错误信息|none|

## GET 登出

GET /user/logout

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|any|false|none|负载数据|none|

*oneOf*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|string|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|integer|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|boolean|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|array|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|object|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|number|false|none||none|

*continued*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» msg|string¦null|false|none|错误信息|none|

## GET 获取当前用户信息

GET /user/whoami

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": {
    "stu_id": "string",
    "name": "string",
    "permissions": [
      "string"
    ]
  },
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|[User](#schemauser)|false|none|负载数据|none|
|»» stu_id|string|true|none|学号|none|
|»» name|string|true|none|姓名|none|
|»» permissions|[string]|true|none|权限|none|
|» msg|string¦null|false|none|错误信息|none|

# 试卷集

## GET 获取试卷集列表

GET /collection

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": [
    {
      "id": 0,
      "name": "string",
      "description": "string",
      "items": [
        {
          "id": 0,
          "date": {
            "typ": null,
            "year": null
          },
          "typ": "string",
          "name": "string",
          "answer": true,
          "page": 0,
          "tags": [
            "string"
          ],
          "comment": "string",
          "md5": "string",
          "categories": [
            "string"
          ]
        }
      ]
    }
  ],
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|[[Collection](#schemacollection)]|false|none|负载数据|none|
|»» id|integer|true|none||none|
|»» name|string|true|none||none|
|»» description|string|true|none|试卷集描述|none|
|»» items|[[Document](#schemadocument)]|true|none|包含的试卷信息|none|
|»»» id|integer|true|none|资料id|none|
|»»» date|object¦null|true|none||为null表示未知年份|
|»»»» typ|string|true|none|资料日期类型|year: 表示在某一年考<br />semester: 表示对应课程是属于哪一年的学期（和 year 的区别主要在于秋季学期）<br />grade: 对应的哪一级学生|
|»»»» year|integer|true|none|年份|none|
|»»» typ|string|true|none|资料类型|final: 期末<br />mid: 期中（含机考）<br />other: 其他|
|»»» name|string|true|none|资料名称|一般是课程名称|
|»»» answer|boolean|true|none|是否有答案|none|
|»»» page|integer|true|none|页数|none|
|»»» tags|[string]|true|none|标签|none|
|»»» comment|string¦null|true|none|说明|none|
|»»» md5|string|true|none|文件md5摘要|none|
|»»» categories|[string]|true|none|分类|比如“A1”，“A2”这种|
|» msg|string¦null|false|none|错误信息|none|

# 搜索

## GET 搜索科目

GET /search

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|key|query|string| 否 |关键字，没有关键字的话|
|typ|query|string| 是 |见 Document 的 typ，可以传递多个|
|page_size|query|integer| 否 |每页的数量，默认为 10|
|page|query|integer| 否 |第几页，默认为 1|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": {
    "subjects": [
      {
        "name": "string",
        "year": 0,
        "count": 0
      },
      {
        "name": "string",
        "year": 0,
        "count": 0
      },
      {
        "name": "string",
        "year": 0,
        "count": 0
      },
      {
        "name": "string",
        "year": 0,
        "count": 0
      },
      {
        "name": "string",
        "year": 0,
        "count": 0
      }
    ],
    "pages": 0
  },
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|object|true|none||none|
|»» subjects|[object]|true|none||none|
|»»» name|string|true|none|科目名称|none|
|»»» year|integer¦null|true|none|最新年份|none|
|»»» count|integer|true|none|包含试卷数量|none|
|»» pages|integer|true|none|总页数|none|
|» msg|string¦null|false|none|错误信息|none|

## GET 搜索对应科目的试卷

GET /search/subject

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|name|query|string| 否 |科目名称|
|typ|query|string| 否 |见 Document 的 typ|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": [
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    },
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    },
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    },
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    },
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    }
  ],
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|[[Document](#schemadocument)]|false|none|负载数据|none|
|»» id|integer|true|none|资料id|none|
|»» date|object¦null|true|none||为null表示未知年份|
|»»» typ|string|true|none|资料日期类型|year: 表示在某一年考<br />semester: 表示对应课程是属于哪一年的学期（和 year 的区别主要在于秋季学期）<br />grade: 对应的哪一级学生|
|»»» year|integer|true|none|年份|none|
|»» typ|string|true|none|资料类型|final: 期末<br />mid: 期中（含机考）<br />other: 其他|
|»» name|string|true|none|资料名称|一般是课程名称|
|»» answer|boolean|true|none|是否有答案|none|
|»» page|integer|true|none|页数|none|
|»» tags|[string]|true|none|标签|none|
|»» comment|string¦null|true|none|说明|none|
|»» md5|string|true|none|文件md5摘要|none|
|»» categories|[string]|true|none|分类|比如“A1”，“A2”这种|
|» msg|string¦null|false|none|错误信息|none|

# 试卷

## POST 下载试卷

POST /document/download

使用工作量证明来防止被攻击。

服务端下发 ticket 值和 zero 值，客户端需要在 ticket 值后面接任意字符串来确保计算出来的 hash 值的前 zero 位为 0.

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|id|query|integer| 否 |要下载试卷的 id|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": {
    "ticket": "string",
    "zero": 0
  },
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|object|false|none|负载数据|none|
|»» ticket|string|true|none||none|
|»» zero|integer|true|none||none|
|» msg|string¦null|false|none|错误信息|none|

## GET 获取试卷下载链接

GET /document/download

可能发生的错误：

* POW_KEY_INVALID：计算的 key 是错误的。同时原来的 ticket 也失效，需要请求 POST /document/download 获得新的 ticket。

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|ticket|query|string| 否 |通过 POST /document/download 获得的 ticket 值|
|key|query|string| 否 |满足条件的字符串。字符串的前缀需要是 ticket|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|string|false|none|负载数据|下载链接|
|» msg|string¦null|false|none|错误信息|none|

## POST 上传试卷

POST /document

可能出现的错误：

* FILE_EXISTED：文件已经存在，此时 data 字段包含对应重复的试卷的信息

> Body 请求参数

```yaml
file: ""
date: ""
typ: ""
name: ""
answer: ""
tags: ""
comment: ""
categories: ""

```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» file|body|string(binary)| 是 |none|
|» date|body|string| 是 |Document 的 date 的 json 格式|
|» typ|body|string| 是 |none|
|» name|body|string| 是 |none|
|» answer|body|boolean| 是 |none|
|» tags|body|string| 是 |Document 的 tags 的 json 格式|
|» comment|body|string| 否 |none|
|» categories|body|string| 是 |Document 的 categories 的 json 格式|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|any|false|none|负载数据|none|

*oneOf*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|string|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|integer|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|boolean|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|array|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|object|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|number|false|none||none|

*continued*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» msg|string¦null|false|none|错误信息|none|

## GET 获取已上传试卷列表

GET /document/pending

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|status|query|string| 是 |见 PendingDocument 的 status|
|page|query|integer| 否 |默认为 1|
|page_size|query|integer| 否 |默认为 10|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": [
    {
      "id": 0,
      "item": {
        "id": 0,
        "date": {
          "typ": "string",
          "year": 0
        },
        "typ": "string",
        "name": "string",
        "answer": true,
        "page": 0,
        "tags": [
          "string"
        ],
        "comment": "string",
        "md5": "string",
        "categories": [
          "string"
        ]
      },
      "status": "string",
      "stu_id": "string",
      "comment": "string",
      "create_time": "string",
      "update_time": "string",
      "target": 0
    }
  ],
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|[[PendingDocument](#schemapendingdocument)]|false|none|负载数据|none|
|»» id|integer|true|none||none|
|»» item|[Document](#schemadocument)|true|none||none|
|»»» id|integer|true|none|资料id|none|
|»»» date|object¦null|true|none||为null表示未知年份|
|»»»» typ|string|true|none|资料日期类型|year: 表示在某一年考<br />semester: 表示对应课程是属于哪一年的学期（和 year 的区别主要在于秋季学期）<br />grade: 对应的哪一级学生|
|»»»» year|integer|true|none|年份|none|
|»»» typ|string|true|none|资料类型|final: 期末<br />mid: 期中（含机考）<br />other: 其他|
|»»» name|string|true|none|资料名称|一般是课程名称|
|»»» answer|boolean|true|none|是否有答案|none|
|»»» page|integer|true|none|页数|none|
|»»» tags|[string]|true|none|标签|none|
|»»» comment|string¦null|true|none|说明|none|
|»»» md5|string|true|none|文件md5摘要|none|
|»»» categories|[string]|true|none|分类|比如“A1”，“A2”这种|
|»» status|string|true|none|审核状态|pending: 正在审核。accepted: 审核通过。rejected: 审核不通过。|
|»» stu_id|string|true|none|提交者|none|
|»» comment|string¦null|true|none|审核意见|当审核不通过时该字段存在，表示不通过的原因|
|»» create_time|string|true|none|上传时间|none|
|»» update_time|string|true|none|更新时间|一般是审核状态更改之后才会更新 update_time|
|»» target|integer¦null|true|none|对应的资料库里的试卷id|批准且系统将试卷处理完毕之后，将会把该试卷添加到正式的资料库中，然后设置该字段。|
|» msg|string¦null|false|none|错误信息|none|

## PUT 更新已上传试卷信息

PUT /document/pending/{id}

只有管理员才能更新。

可能出现的错误：

* FILE_SIZE_LIMIT_EXCEEDED：文件大小超出限制
* FILE_EXISTED：文件已经存在，此时 data 字段包含对应重复的试卷的信息

> Body 请求参数

```yaml
file: ""
date: ""
typ: ""
name: ""
answer: ""
tags: ""
comment: ""
categories: ""

```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|id|path|integer| 是 |none|
|body|body|object| 否 |none|
|» file|body|string(binary)| 否 |不附加就是不更改文件|
|» date|body|string| 是 |Document 的 date 的 json 格式|
|» typ|body|string| 是 |none|
|» name|body|string| 是 |none|
|» answer|body|boolean| 是 |none|
|» tags|body|string| 是 |Document 的 tags 的 json 格式|
|» comment|body|string| 否 |none|
|» categories|body|string| 是 |Document 的 categories 的 json 格式|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|any|false|none|负载数据|none|

*oneOf*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|string|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|integer|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|boolean|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|array|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|object|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|number|false|none||none|

*continued*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» msg|string¦null|false|none|错误信息|none|

## GET 评审已上传试卷

GET /document/pending/{id}

只有管理员才能审批。只能审批为 pending 状态的。

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|id|path|integer| 是 |none|
|status|query|string| 是 |accepted 或是 rejected|
|comment|query|string| 否 |如果是 rejected，需要提供该字段，表示拒绝理由。|

> 返回示例

> 200 Response

```json
{
  "status": "string",
  "data": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» status|string|true|none|请求的状态信息|成功则为 OK，失败则为相应的错误代码。失败时 msg 保存可选的可读信息，data 里也可存放附加的字段。|
|» data|any|false|none|负载数据|none|

*oneOf*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|string|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|integer|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|boolean|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|array|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|object|false|none||none|

*xor*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|»» *anonymous*|number|false|none||none|

*continued*

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» msg|string¦null|false|none|错误信息|none|

# 数据模型

<h2 id="tocS_Document">Document</h2>

<a id="schemadocument"></a>
<a id="schema_Document"></a>
<a id="tocSdocument"></a>
<a id="tocsdocument"></a>

```json
{
  "id": 0,
  "date": {
    "typ": "string",
    "year": 0
  },
  "typ": "string",
  "name": "string",
  "answer": true,
  "page": 0,
  "tags": [
    "string"
  ],
  "comment": "string",
  "md5": "string",
  "categories": [
    "string"
  ]
}

```

### 属性

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|id|integer|true|none|资料id|none|
|date|object¦null|true|none||为null表示未知年份|
|» typ|string|true|none|资料日期类型|year: 表示在某一年考<br />semester: 表示对应课程是属于哪一年的学期（和 year 的区别主要在于秋季学期）<br />grade: 对应的哪一级学生|
|» year|integer|true|none|年份|none|
|typ|string|true|none|资料类型|final: 期末<br />mid: 期中（含机考）<br />other: 其他|
|name|string|true|none|资料名称|一般是课程名称|
|answer|boolean|true|none|是否有答案|none|
|page|integer|true|none|页数|none|
|tags|[string]|true|none|标签|none|
|comment|string¦null|true|none|说明|none|
|md5|string|true|none|文件md5摘要|none|
|categories|[string]|true|none|分类|比如“A1”，“A2”这种|

<h2 id="tocS_User">User</h2>

<a id="schemauser"></a>
<a id="schema_User"></a>
<a id="tocSuser"></a>
<a id="tocsuser"></a>

```json
{
  "stu_id": "string",
  "name": "string",
  "permissions": [
    "string"
  ]
}

```

### 属性

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|stu_id|string|true|none|学号|none|
|name|string|true|none|姓名|none|
|permissions|[string]|true|none|权限|none|

<h2 id="tocS_Collection">Collection</h2>

<a id="schemacollection"></a>
<a id="schema_Collection"></a>
<a id="tocScollection"></a>
<a id="tocscollection"></a>

```json
{
  "id": 0,
  "name": "string",
  "description": "string",
  "items": [
    {
      "id": 0,
      "date": {
        "typ": "string",
        "year": 0
      },
      "typ": "string",
      "name": "string",
      "answer": true,
      "page": 0,
      "tags": [
        "string"
      ],
      "comment": "string",
      "md5": "string",
      "categories": [
        "string"
      ]
    }
  ]
}

```

### 属性

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|id|integer|true|none||none|
|name|string|true|none||none|
|description|string|true|none|试卷集描述|none|
|items|[[Document](#schemadocument)]|true|none|包含的试卷信息|none|

<h2 id="tocS_PendingDocument">PendingDocument</h2>

<a id="schemapendingdocument"></a>
<a id="schema_PendingDocument"></a>
<a id="tocSpendingdocument"></a>
<a id="tocspendingdocument"></a>

```json
{
  "id": 0,
  "item": {
    "id": 0,
    "date": {
      "typ": "string",
      "year": 0
    },
    "typ": "string",
    "name": "string",
    "answer": true,
    "page": 0,
    "tags": [
      "string"
    ],
    "comment": "string",
    "md5": "string",
    "categories": [
      "string"
    ]
  },
  "status": "string",
  "stu_id": "string",
  "comment": "string",
  "create_time": "string",
  "update_time": "string",
  "target": 0
}

```

### 属性

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|id|integer|true|none||none|
|item|[Document](#schemadocument)|true|none||none|
|status|string|true|none|审核状态|pending: 正在审核。accepted: 审核通过。rejected: 审核不通过。|
|stu_id|string|true|none|提交者|none|
|comment|string¦null|true|none|审核意见|当审核不通过时该字段存在，表示不通过的原因|
|create_time|string|true|none|上传时间|none|
|update_time|string|true|none|更新时间|一般是审核状态更改之后才会更新 update_time|
|target|integer¦null|true|none|对应的资料库里的试卷id|批准且系统将试卷处理完毕之后，将会把该试卷添加到正式的资料库中，然后设置该字段。|

