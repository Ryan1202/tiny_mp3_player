
# My Tiny MP3 Player

一个用Rust编写的简单MP3播放器，用于学习MP3解码。目前仅支持单声道解码。

## 编译

```bash
cargo build --release
```

## 使用

```bash
tiny_mp3_player --input_file ./test.mp3 # 播放MP3文件
```
```bash
tiny_mp3_player --input_file ./test.mp3 --debug all # 播放MP3文件并输出每一帧的所有信息
```

> 使用main.rs中的test可以将解码后的PCM数据输出到wav文件。

## 脚本

- 根目录下的`compare.py`可以用来对比两个wav文件的差异。

- `src/huffman/change.py`可以用来把`tbl.txt`中的哈夫曼表数据转换成数组存到`out.txt`中。

- `src/decode/cal.py`用来计算`imdct.rs`中所需的`SINE_BLOCK`数据



## 依赖

- rodio: 音频播放

- thiserror: 错误输出文本处理

