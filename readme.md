# TradeSession
### 用途
用于判断某时间点或段上，某个品种是否处于可交易状态

### 使用方法

参考 tradesessionpy/ex1.py  
rust/c++/python使用方法类似  


### Python 绑定
- 切换到需要的虚拟环境  
conda activate your-env-name
- 生成/更新pyi, 可能需要把LD_LIBARY_PATH指向你env所在的lib目录  
cargo run --bin stub_gen  或者  
LD_LIBARY_PATH=???env/lib  cargo run --bin stub_gen   
- 进入tradesessionpy子目录  
cd tradesessionpy
- 安装maturin  
https://github.com/PyO3/maturin  
conda install maturin 
或者 pip install maturin  
- 编译该虚拟环境对应python版本的whl包,用以分发然后手动安装  
maturin build --release
- 或者,直接为当前虚拟环境安装whl包  
maturin develop --release

### C++绑定
- 编译release版本通过
- 复制target/cxxbridge/{rust, tradesessionpp}及之下的所有.h和.cc文件  
  包括cxx.h, ???.rs.h, ???.rs.cc  
- 下载cxx.cc文件,   
  https://raw.githubusercontent.com/dtolnay/cxx/refs/heads/master/src/cxx.cc
- 复制target/release下面的tradesessionpp.{dll,lib}文件, linux下则为libtradesessionpp.so
- c++封装文件: 在tradesessionpp/wrapper目录下，复制到c++项目


### ShiftedTime的计算
因为barbuilder项目需要使用到tradession crate  
从chrono::NaiveTime计算ShiftedTime时，有一个问题，跟我们的K线切分方式相关,  
切分k线时，我们使用左开右闭区间(]，整点时间是属于前一个周期的，  
比如上午休市时的10:15:00, 属于前一个bar，比如收盘时15:00:00,它属于上一个bar  
比如商品期货，早上的第一个一分钟bar,  
如果不含集合竞价，它是`[9:00:00～9:01:00]`, 第二个是`(9:01:00~9:02:00]`  
如果包含集合竞价，它是`[8:59:00～9:01:00]`, 第二个是`(9:01:00~9:02:00]`  

计算ShiftedTime如果丢弃毫秒的话，就会有问题，   
比如2025-07-23 00:00:00和2025-07-23 00:00:00.500,  
计算出来的ShiftedTime秒数是一样的，500ms的差异被弄丢了  
但实际上，前者是上一个bar的结束，后者是新一个bar的开始  
所以，只要毫秒数非零，就应该归于后一秒，
