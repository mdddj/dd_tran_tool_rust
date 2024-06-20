use baidu_trans::aio::Client;
use baidu_trans::config::Config;
use baidu_trans::lang::Lang;
use baidu_trans::model::CommonResult;
use clap::Parser;
use serde::Deserialize;
use tokio::runtime::Runtime;

use std::borrow::{Borrow, BorrowMut};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::sync::{OnceCell, Semaphore};
use tokio::time::{self, Instant};

pub static BAIDU_CONFIG: OnceCell<App> = OnceCell::const_new();

struct App {
    pub config: MyConfig,
}

impl App {
    ///创建一个客户端
    pub fn create_baidu_client(&self) -> Client {
        Client::new(Config::new(
            self.config.baiduId.clone(),
            self.config.baiduKey.clone(),
        ))
    }
}

async fn get_baidu_config() {
    BAIDU_CONFIG
        .get_or_init(|| async {
            let arc_config = read_config().await.expect("读取配置文件失败");
            let is_dir = directory_exists(&arc_config.propertiesFileDir);
            if !is_dir {
                panic!("目录不存在:{:?}", arc_config.propertiesFileDir);
            }
            let app = App { config: arc_config };
            app
        })
        .await;
}

async fn get_app() -> &'static App {
    BAIDU_CONFIG.get().expect("读取配置失败.")
}

// 将字符串转换为Lang枚举的函数
pub fn str_to_lang(lang_str: &str) -> Result<Lang, String> {
    match lang_str {
        "auto" => Ok(Lang::Auto),
        "zh" => Ok(Lang::Zh),
        "en" => Ok(Lang::En),
        "yue" => Ok(Lang::Yue),
        "wyw" => Ok(Lang::Wyw),
        "ja" => Ok(Lang::Jp),
        "ko" => Ok(Lang::Kor),
        "fra" => Ok(Lang::Fra),
        "fr" => Ok(Lang::Fra),
        "spa" => Ok(Lang::Spa),
        "th" => Ok(Lang::Th),
        "ara" => Ok(Lang::Ara),
        "ar" => Ok(Lang::Ara),
        "ru" => Ok(Lang::Ru),
        "pt" => Ok(Lang::Pt),
        "de" => Ok(Lang::De),
        "it" => Ok(Lang::It),
        "el" => Ok(Lang::El),
        "nl" => Ok(Lang::Nl),
        "pl" => Ok(Lang::Pl),
        "bul" => Ok(Lang::Bul),
        "est" => Ok(Lang::Est),
        "dan" => Ok(Lang::Dan),
        "fin" => Ok(Lang::Fin),
        "cs" => Ok(Lang::Cs),
        "rom" => Ok(Lang::Rom),
        "slo" => Ok(Lang::Slo),
        "swe" => Ok(Lang::Swe),
        "hu" => Ok(Lang::Hu),
        "hk" => Ok(Lang::Cht),
        "vie" => Ok(Lang::Vie),

        _ => Err(format!("Unknown language code: {}", lang_str)),
    }
}

#[derive(Deserialize, Debug, Clone)]
struct MyConfig {
    baiduId: String,
    baiduKey: String,
    propertiesFileDir: String,
    filename: String,
    defaultfilename: String,
    defaultLang: String,
    suportLangs: Vec<String>,
}

struct TranTask {
    text: String,
    to_lang: String,
}

impl TranTask {
    ///执行翻译
    async fn run(&self, key: &str) {
        let config = get_app().await.config.clone();
        let dir = config.propertiesFileDir;
        let filename = config.filename;
        let r = tr(self.text.as_str(), self.to_lang.as_str()).await;
        match r {
            Ok(r) => {
                let result_comment = r.result.trans_result;
                match result_comment {
                    Some(tr_result) => {
                        if let Some(std) = tr_result.first() {
                            write_key_value_to_file(
                                &dir,
                                format!("{}_{}", &filename, r.to).as_str(),
                                key,
                                &std.dst,
                            );
                        }
                    }
                    None => {
                        println!("翻译失败:{},语言:{:?}", r.result.error_msg.unwrap(), r.to);
                    }
                }
            }
            Err(e) => println!("翻译失败:{:?}", e),
        }
    }
}

///检测文件是否存在
fn directory_exists(relative_path: &str) -> bool {
    // 获取当前执行命令的目录
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // 构建绝对路径
    let absolute_path = current_dir.join(relative_path);

    // 检查路径是否存在且是一个目录
    absolute_path.is_dir()
}

///初始化配置文件
fn init() {
    let mut file = File::create(PathBuf::from(".").join(".ddtr.json")).expect("初始化配置文件失败");

    let json = r#"
        {
            "baiduId":"",
            "baiduKey":"",
            "propertiesFileDir": "./src/main/resources/messages",
            "filename": "pluginBundle",
            "defaultfilename":"pluginBundle",
            "defaultLang":"zh",
            "suportLangs":[
                "en",
                "hk",
                "ja",
                "ko"
            ]
        }
        "#;

    file.write_all(json.as_bytes()).expect("写配置文件失败");

    println!("初始化文件成功");
}

///读取.ddtr.json配置文件
async fn read_config() -> Result<MyConfig, Box<dyn std::error::Error>> {
    let path = Path::new(".ddtr.json");
    let mut file = tokio::fs::File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let model: MyConfig = serde_json::from_str(&content)?;
    return Ok(model);
}

#[derive(Debug)]
struct MyResult {
    txt: String,
    to: String,
    result: CommonResult,
}

///翻译函数
async fn tr(t: &str, to: &str) -> Result<MyResult, String> {
    let app = get_app().await;
    let config = &app.config;
    let default_from = config.defaultLang.clone();
    let from_lang =
        str_to_lang(&default_from).expect(format!("不支持的翻译:{default_from}").as_str());
    let to_lang = str_to_lang(&to).expect(format!("不支持转换的翻译:{to}").as_str());
    let client = app.create_baidu_client();
    client.lang(from_lang, to_lang);
    let resp = client.translate(t).await;
    match resp {
        Ok(e) => Ok(MyResult {
            txt: t.to_string(),
            to: to.to_string(),
            result: e,
        }),
        Err(er) => Err(format!("翻译出错:{:?}", er)),
    }
}

///开始批量翻译
async fn process_tr_task(tran_txt: &str, key: &str) {
    let qps = 1;
    let interval = Duration::from_secs(1) / qps as u32;
    let semaphore = Arc::new(Semaphore::new(qps));
    let app = get_app().await;
    let app_config = app.config.clone();
    let langs = app_config.suportLangs;
    let dir = app_config.propertiesFileDir;
    let filename = app_config.filename;
    let defualt_filename = filename.clone();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        for to_lang in langs {
            let task = TranTask {
                text: tran_txt.clone().to_string(),
                to_lang: to_lang,
            };
            task.run(key).await;
        }
    });

    write_key_value_to_file(&dir, &defualt_filename, key, tran_txt);
}

fn write_key_value_to_file(dir: &str, file_name: &str, key: &str, value: &str) {
    // 构建文件路径
    let file_path = format!("{}/{}.properties", dir, file_name);

    let file_path_2 = &file_path;
    // 写入翻译结果 key=翻译结果
    let properties_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path_2);
    match properties_file {
        Ok(_file) => {
            let mut writer = BufWriter::new(_file);
            let wr = writeln!(writer, "\n{}={}", key, value);
            match wr {
                Ok(_) => {}
                Err(e) => println!("写入翻译结果失败:{},翻译结果:{},", e, value),
            }
            let rfr = writer.flush();
            match rfr {
                Ok(_) => {
                    println!("写入成功:{:?},   \t\t{}={}", file_path_2, key, value);
                }
                Err(e) => println!("写入翻译保存结果失败,翻译结果:{},原因:{:?}", value, e),
            }
        }
        Err(e) => println!("获取文件失败:{:?}", e),
    }
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct MyArgs {
    /// 要执行的操作
    method: String,

    ///要翻译的文本
    #[arg(short, long)]
    tran: Option<String>,

    ///键值对的键,key=翻译结果
    #[arg(short, long)]
    key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = MyArgs::parse();

    // let args: Vec<String> = std::env::args().collect();

    let method = args.method;

    if method == "init" {
        init();
    }
    if method == "tran" {
        get_baidu_config().await;
        let tran = args.tran.expect("请输入要翻译的词");
        let key = args.key.expect("请输入建议对的键");
        //翻译的文本
        let start = Instant::now();
        process_tr_task(&tran, &key).await;
        let end = Instant::now();
        let duration = end.duration_since(start);
        println!("代码运行耗时: {:?}秒", duration.as_secs_f64());
    }

    Ok(())
}
