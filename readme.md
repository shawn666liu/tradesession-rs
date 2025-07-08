

### Python 绑定
- 生成/更新pyi  
cargo run --bin stub_gen
- 进入???py子目录  
cd ???py
- 切换到需要的虚拟环境  
conda activate your-env-name
- 安装maturin  
https://github.com/PyO3/maturin  
conda install maturin 
或者 pip install maturin  
- 编译该虚拟环境对应python版本的whl包,用以分发然后手动安装  
maturin build --release
- 或者,直接为当前虚拟环境安装whl包  
maturin develop --release

### C++绑定
