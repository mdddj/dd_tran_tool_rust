# dd_tran_tool_rust

# 安装

```bash
cargo install dd_tran_tool
```

# 1.初始化

在idea插件项目的根目录下执行
```bash
dd_tran_tool init
```

会生成一个配置文件`.ddtr.json`

```json
    {
        "baidu_id":"",
        "baidu_key":"",
        "properties_file_dir": "./src/main/resources/messages",
        "filename": "pluginBundle",
        "defaultfilename":"pluginBundle",
        "default_lang":"zh",
        "suport_langs":[
            "en",
            "hk",
            "ja",
            "ko"
        ]
    }
```
`baidu_id`: 百度翻译api id

`baidu_key`: 百度翻译api key

`properties_file_dir`: 插件的国际化配置目录,相对路径

`filename`: 文件名的前缀

`defaultfilename`: 默认的文件名

`default_lang`: 翻译的文本语言

`suport_langs`: 要翻译的文本语言




# 2. 翻译

```bash
dd_tran_tool tran --tran 要翻译的文本 --key 键值对的键
```


例子
```
dd_tran_tool tran --tran 测试 --key test
```

会在文件`/src/main/resources/messages/pluginBundle.properties`末尾添加
```
test=测试
```
会在文件`/src/main/resources/messages/pluginBundle_en|hk|ja|ko.properties`末尾添加
```
test=翻译后的对应语言结果
```