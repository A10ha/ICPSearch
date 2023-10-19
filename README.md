# ICPSearch
![image](https://github.com/A10ha/ICPSearch/assets/60035496/aac89a45-388a-4ba7-8228-89cad1854470)

**ICP Lookup Tool** 是一个基于Rust编写的命令行工具，主要用于查找并获取网站域名的ICP备案信息。

## 项目功能

1. 通过域名、URL或者企业名（全程）查找ICP备案信息。你可以输入指定的域名或者企业名（全程），然后获取相应的备案信息。

   ```bash
   ICPSearch.exe -d yourdomain.com
   ```
![image](https://github.com/A10ha/ICPSearch/assets/60035496/ab51e053-fc2c-4736-9ddc-a59fa87ae734)

2. 批量处理多个域名、URL和企业名（全称）。你可以在文本文件中列出需要查找的多个域名或者企业名（全称），然后通过该工具一次性处理这些域名和企业名（全称），并获取相应的备案信息。

   ```bash
   ICPSearch.exe -f domains.txt
   ```
![image](https://github.com/A10ha/ICPSearch/assets/60035496/b4237cf1-af88-40cf-9b42-e96d23ee6e37)
![image](https://github.com/A10ha/ICPSearch/assets/60035496/f83b1206-4da1-43fd-9109-a6e3361fc7f6)

## 数据输出

所有的结果将会被打印到console，同时写入到名为result.txt的文件中。

## 注意事项

该工具的预设并发数量为50，请结合实际情况和目标服务器的承受能力来调节该值。过大的并发请求可能对目标服务器产生压力。

## 许可证

Apache License 2.0
