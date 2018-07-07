extern crate iron;
extern crate url;
extern crate tcpgen;
extern crate image;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use tcpgen::{TCPList, TCP};
use std::fs::File;
use std::path::Path;
use std::io::Read;

fn main() {
    let tcp_list = TCPList::new("./");

    Iron::new(move |request: &mut Request| {
        let resource = request.url.path().pop().unwrap();
        if resource.is_empty() {
            let tcp = if let Some(query) = request.url.query() {
                let mut tcp: TCP = Default::default();
                for arg in query.split('&') {
                    let arg = url::percent_encoding::percent_decode(arg.as_bytes()).decode_utf8_lossy().to_string();
                    let mut split = arg.splitn(2, '=');
                    let key = split.next().unwrap_or("").to_lowercase();
                    match key.as_ref() {
                        "types" => {
                            if let Some(types) = split.next() {
                                let mut types = types.split(',').map(|value| value.to_string()).collect();
                                tcp.types.append(&mut types);
                            }
                        },
                        "anomalies" => {
                            if let Some(anomalies) = split.next() {
                                let mut anomalies = anomalies.split(',').map(|value| value.to_string()).collect();
                                tcp.anomalies.append(&mut anomalies);
                            }
                        },
                        "conditions" => {
                            if let Some(conditions) = split.next() {
                                let mut conditions = conditions.split(',').map(|value| value.to_string()).collect();
                                tcp.conditions.append(&mut conditions);
                            }
                        },
                        "modifiers" => {
                            if let Some(modifiers) = split.next() {
                                let mut modifiers = modifiers.split(',').map(|value| value.to_string()).collect();
                                tcp.modifiers.append(&mut modifiers);
                            }
                        },
                        "designer" => {
                            tcp.designer = true;
                        }
                        _ => println!("Unknown parameter: {}", key)
                    }
                }
                if tcp.types.is_empty() || tcp.types[0].is_empty() {
                    tcp.types = vec!["typeless".to_string()];
                }
                tcp
            } else {
                tcp_list.gen()
            };

            let mut filename = {
                let mut sorted_types = tcp.types.clone();
                sorted_types.sort();
                sorted_types.join(".")
            };
            if !tcp.anomalies.is_empty() {
                let mut sorted_anomalies = tcp.anomalies.clone();
                sorted_anomalies.sort();
                filename = format!("{}-a_{}", filename, sorted_anomalies.join(".").to_lowercase())
            }
            if !tcp.conditions.is_empty() {
                let mut sorted_conditions = tcp.conditions.clone();
                sorted_conditions.sort();
                filename = format!("{}-c_{}", filename, sorted_conditions.join(".").to_lowercase())
            }
            if !tcp.modifiers.is_empty() {
                let mut sorted_modifiers = tcp.modifiers.clone();
                sorted_modifiers.sort();
                filename = format!("{}-m_{}", filename, sorted_modifiers.join(".").to_lowercase())
            }
            filename += ".png";
            let path = format!("bases/{}", filename);
            let path = Path::new(&path);

            let mut page = format!("<html><head><title>{}</title><link rel=\"icon\" href=\"favicon.ico\"></head><div style=\"margin: 0 auto; text-align: center\"><h2>{}</h2>", tcp, tcp);
            if path.exists() {
                if tcp.designer {
                    filename += "?designer";
                }
                page += &format!("<p><img src=\"{}\" alt=\"{}\"></p>", filename, filename);
            } else {
                page += &format!("<br><p>No base named {}</p>", filename);
                for tcp_type in tcp.types {
                    let filename = format!("{}.png", tcp_type);
                    let path = format!("bases/{}", filename);
                    let path = Path::new(&path);
                    if path.exists() {
                        page += &format!("<div style=\"width: 600px; display: inline-block\"><img src=\"{}\" alt=\"{}\"><p>{}</p></div>", filename, filename, tcp_type);
                    }
                }
            }
            page += "</div></html>";
            Ok(Response::with((ContentType::html().0, status::Ok, page)))
        } else {
            let resource = url::percent_encoding::percent_decode(resource.as_bytes()).decode_utf8_lossy().to_string();
            if resource == "info" {
                Ok(Response::with((status::Ok, format!("{:#?}", tcp_list))))
            } else if resource == "baseless" {
                let mut response = String::new();
                for (category, list) in tcp_list.types.iter() {
                    let mut baseless = Vec::new();
                    for tcp_type in list.iter() {
                        let filename = format!("{}.png", tcp_type);
                        let path = format!("bases/{}", filename);
                        let path = Path::new(&path);
                        if !path.exists() {
                            baseless.push(tcp_type.clone())
                        }
                    }
                    if !baseless.is_empty() {
                        response += &format!("{} {{\n    {}\n}}\n", category, baseless.join("\n    "));
                    }
                }
                Ok(Response::with((status::Ok, response)))
            } else if let Ok(mut file) = File::open(format!("bases/{}", resource)) {
                let mut data = Vec::new();
                file.read_to_end(&mut data).unwrap();
                if request.url.query().is_some() {
                    let mut buffer = image::load_from_memory_with_format(&data, image::PNG).unwrap().to_rgba();
                    let (width, height) = buffer.dimensions();
                    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                        let x = width as f32 / 2.0 - x as f32;
                        let y = height as f32 / 2.0 - y as f32;
                        let angle0 = x.atan2(y) * 180.0 / std::f32::consts::PI + 360.0;
                        let distance = (x * x + y * y).sqrt();
                        if pixel[3] == 0 &&
                            angle0 % 10.0 < 5.0 &&
                            distance % distance.sqrt() > distance.sqrt() / (distance.sqrt() / 3.0).sqrt()
                             {
                            pixel.data = [0, 0, 0, 0xff]
                        }
                    }
                    data.clear();
                    image::ImageRgba8(buffer).save(&mut data, image::PNG).unwrap();
                }
                Ok(Response::with((ContentType::png().0, status::Ok, data)))
            } else {
                println!("{}", resource);
                Ok(Response::with((status::NotFound, format!("{} doesn't exist", resource))))
            }
        }
    }).http("localhost:17080").unwrap();
}
