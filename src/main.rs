// 快速执行 adb shell dumpsys dropbox ，并找到最近的 crash 命令
extern crate clap;

use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use alfred::{Item, Modifier};
use clap::{Arg, arg};
use clap::Parser;
use walkdir::WalkDir;
use serde::Serialize;
use serde::Deserialize;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    fun_name: String,

    #[arg(short, long, value_name = "device_id")]
    device_id: Option<String>,

    #[arg(short, long, value_name = "workspace")]
    workspace: Option<String>,

}

#[derive(Deserialize, Debug, Serialize)]
struct Config {
    workspace: String,
}

fn read_config() -> Config {
    let config: Config = {
        let mut config_file = get_config_file();
        let mut config_text = String::new();
        config_file.read_to_string(&mut config_text).expect("TODO: panic message");
        toml::from_str(&config_text).unwrap()
    };
    return config;
}

fn get_config_file() -> File {
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("config.toml").expect("error create file");
    return config_file;
}

fn write_config(config: Config) {
    let mut config_file = get_config_file();
    let string = toml::to_string(&config).unwrap();
    config_file.write(&string.as_bytes()).unwrap();
}

fn set_workspace(args: &Args) {
    if let Some(workspace) = &args.workspace {
        write_config(Config {
            workspace: workspace.clone(),
        })
    }
}

fn get_workspace() -> String {
    let config = read_config();
    // 所有权交出去了 那我 config 咋整？
    return config.workspace;
}

fn main() {
    let args = Args::parse();

    set_workspace(&args);

    // println!("fun_name = {}", args.fun_name);
    if args.fun_name == "crash" {
        if let Some(device_id) = args.device_id {
            find_crash(device_id);
        }
    } else if args.fun_name == "devices" {
        find_devices()
    } else if args.fun_name == "open_debug" {
        if let Some(device_id) = args.device_id {
            open_douyin_debug(device_id)
        }
    } else if args.fun_name == "did" {
        // 展示 did
        show_my_did()
    } else if args.fun_name == "usb" {
        restart_usb()
    }
}

fn simple_write_alfred_output(title: &str, arg: &str, sub_title: &str) {
    alfred::json::Builder::with_items(&[
        alfred::ItemBuilder::new(title)
            .arg(arg)
            .subtitle(sub_title)
            .into_item()
    ]).write(io::stdout()).expect("error");
}

fn restart_usb() {
    let output = Command::new("adb").arg("usb").output().expect("").stdout;
    let output = String::from_utf8(output).expect("转义失败");
    simple_write_alfred_output("重启 usb成功", &output, "");
}

fn show_my_did() {
    alfred::json::Builder::with_items(&[
        alfred::ItemBuilder::new("pix 6")
            .arg("1148358348243358")
            .subtitle("1148358348243358")
            .into_item(),
        alfred::ItemBuilder::new("小米手机")
            .arg("954797464765992")
            .subtitle("954797464765992")
            .into_item(),
        alfred::ItemBuilder::new("华为手机")
            .arg("3069463893389480")
            .subtitle("3069463893389480")
            .into_item(),
        alfred::ItemBuilder::new("模拟器")
            .arg("266498979143011")
            .subtitle("266498979143011")
            .into_item(),
    ]).write(io::stdout()).expect("ff")
}

fn open_douyin_debug(device_id: String) {
    // adb shell am broadcast -a com.ss.android.ugc.aweme.util.crony.action_open_debug
    let output = Command::new("adb").args(["-s", &device_id, "shell", "am", "broadcast", "-a", "com.ss.android.ugc.aweme.util.crony.action_open_debug"]).output().expect("ddd").stdout;
    let output = String::from_utf8(output).expect("转义失败");
    simple_write_alfred_output("打开抖音 debug 页面", &output, "")
}

fn find_devices() {
    let output = Command::new("adb").arg("devices").output().expect("执行 adb devices").stdout;
    let output = String::from_utf8(output).expect("转义失败");
    let mut all_devices: Vec<&str> = output.split("\n").filter(|x| {
        x.contains("device") && !x.contains("List of devices attached")
    }).collect();
    let all_devices: Vec<_> = all_devices.iter_mut().map(|device| {
        return device.replace("device", "").trim().to_string();
    }).collect();
    let mut alfred_items: Vec<Item> = Vec::new();
    for device in all_devices {
        alfred_items.push(alfred::ItemBuilder::new(device.clone())
            .arg(device.clone())
            .subtitle("设备 id")
            .into_item())
    }
    alfred::json::Builder::with_items(&alfred_items)
        .write(io::stdout()).expect("error")
}

fn find_crash(device_id: String) {
    let output = Command::new("adb").arg("-s").arg(device_id.clone()).arg("shell").arg("dumpsys").arg("dropbox").output().expect("执行异常");

    let all_string = String::from_utf8(output.stdout).expect("转义失败");
    let last_crash_line: Vec<&str> = all_string.split("\n").filter(|x| {
        x.contains("data_app_crash")
    }).collect();

    match last_crash_line.last() {
        None => simple_write_alfred_output("没有找到异常", "", ""),
        Some(last) => {
            let split: Vec<&str> = last.split("(").collect();
            // 取出来第一个
            let target = split.first();
            match target {
                None => simple_write_alfred_output("最近没有异常", "", ""),
                Some(first) => {
                    let detail_out_put = Command::new("adb").arg("-s").arg(device_id.clone()).arg("shell").arg("dumpsys").arg("dropbox").arg("--print").arg(first).output().expect("执行异常");
                    let detail_out_put_string = String::from_utf8(detail_out_put.stdout).expect("转义失败");
                    simple_write_alfred_output("崩溃堆栈", &detail_out_put_string, &detail_out_put_string)
                }
            }
        }
    }
}