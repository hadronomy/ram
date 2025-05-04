# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.1]

### 🚀 Features

- *(lsp)* Use stdio for as lsp transport - ([71fdd34](https://github.com/hadronomy/ram/commit/71fdd3444f6c53a71cd2fb9e8c5d291f0e0be90d))
- Add array access support for the `hir` - ([cacea79](https://github.com/hadronomy/ram/commit/cacea79090a9dbe3ebd839ea38f3f8f2adcd34e7))
- Add cfg visualization - ([e265e6b](https://github.com/hadronomy/ram/commit/e265e6bedff81610bcbc182c2831be98148a8c38))
- Add pipeline visualization in mermaid - ([fad18dc](https://github.com/hadronomy/ram/commit/fad18dc9514d8805b591a3ff35dacd0417dd6fb4))
- Add pipeline visualization exports - ([9f4b57c](https://github.com/hadronomy/ram/commit/9f4b57c017b8145ddc267359d353cb295553e95b))
- Constant propagation analysis - ([e1bd041](https://github.com/hadronomy/ram/commit/e1bd04151f71d57818f97755acf41466554ce930))
- Print the control flow graph in the validate command for testing purposes - ([af5c0a5](https://github.com/hadronomy/ram/commit/af5c0a55d34efb200fa48032150b478f6d0b237e))
- Add analysis passes to `validate` command - ([0c8ba19](https://github.com/hadronomy/ram/commit/0c8ba19731af795bdd0c45c0eb458b141f9b0b10))
- Use VecInput and VecOutput when running the program - ([7c5ae15](https://github.com/hadronomy/ram/commit/7c5ae15606b36a78eb12ed5857164a5e491eb841))
- Start basic `vm` implementation - ([b17b419](https://github.com/hadronomy/ram/commit/b17b4198e0cca539c39b8ab3ff854df8b624ff06))
- Use `cstree` instead of `rowan` for the `CST` - ([87e29f0](https://github.com/hadronomy/ram/commit/87e29f08968bd65756f734c210272455de204b21))
- Enable `show_related_errors_as_nested` for miette - ([f831782](https://github.com/hadronomy/ram/commit/f831782cc3c59a86fc724ed4a40bc372840ff316))
- Setup `human_panic` - ([bc8fcda](https://github.com/hadronomy/ram/commit/bc8fcda38a6ea9e614e58f55f192567a9630069c))
- Wayyy better tracing and logging handling - ([4761eb5](https://github.com/hadronomy/ram/commit/4761eb5cc7e3a4c955383da6893e6a419224204b))
- Improve version handling for cli and lsp - ([5c50514](https://github.com/hadronomy/ram/commit/5c50514b30ee39586615bdec50d639895e92d762))
- Add `version` command - ([c1fb19b](https://github.com/hadronomy/ram/commit/c1fb19bf624128f7bf8dab4018c0ef61599a73ec))
- Start new parser implementation in `ram-parser` crate - ([f3209d4](https://github.com/hadronomy/ram/commit/f3209d4130f7886617d38c71115bcf53cd18c5d4))
- Add syntax kind - ([4e89fce](https://github.com/hadronomy/ram/commit/4e89fce8bcf3e457901e0891c3e89afbd56b9404))
- Make lsp server autorestart if it crashes - ([fe6918a](https://github.com/hadronomy/ram/commit/fe6918a388b2e107b492d3b9cef3a2400369d56c))
- Make the lsp runtime agnostic - ([067f8cf](https://github.com/hadronomy/ram/commit/067f8cf74ae4252c00351de3e53af823d3f574d6))
- Add example lsp server - ([fd170f1](https://github.com/hadronomy/ram/commit/fd170f1a2a161fc54c1697b71c1cf989ef531bd1))
- Add diagnostic code to `ParserError` - ([888087b](https://github.com/hadronomy/ram/commit/888087b00d3973e637ce90744159255501fb8bfe))
- Migrate to `miette` for parser errors and improve reporting - ([ad06b48](https://github.com/hadronomy/ram/commit/ad06b481e314c9eebdb914d996849e5931abe6ed))
- Clean up the cli and remove hardcoded strings - ([22cf58f](https://github.com/hadronomy/ram/commit/22cf58f0a880e95ec763bb65a9718f29329af59d))
- Add basic parser in rust - ([c4a9973](https://github.com/hadronomy/ram/commit/c4a9973aff0d5315e9aba99df73ca4801e13102c))
- Add `lsp` command - ([7ee2c35](https://github.com/hadronomy/ram/commit/7ee2c358915c9127a3b2d4a358beca80480b2714))
- Add rust crates boilerplate - ([dc8f608](https://github.com/hadronomy/ram/commit/dc8f608dc3f42e87ee7f6ab3733ecf8e346d1c32))

### 🐛 Bug Fixes

- *(lsp)* Server crashing when invalid mirror log path given - ([1886ddd](https://github.com/hadronomy/ram/commit/1886ddd6e6c8c8d7b4925b8096e12916bba2afee))
- Wrong version in cli config - ([d7727be](https://github.com/hadronomy/ram/commit/d7727be63c8d2a8ea62474119cc2c24378583895))
- Colorization not being detected properly - ([b2bcbd7](https://github.com/hadronomy/ram/commit/b2bcbd77a9112483eb4991da9cfa531277c38bd5))
- Print errors to the stderr - ([f9e472a](https://github.com/hadronomy/ram/commit/f9e472afb0b771f968c93a27a865800b17b945c8))
- Missing binary - ([92aea25](https://github.com/hadronomy/ram/commit/92aea25f62e0c383c83d3c650406d0af0cbd884f))

### 🚜 Refactor

- Move tracing setup to its own module - ([cedda89](https://github.com/hadronomy/ram/commit/cedda8901991119b57289a8fdef625eb3d52597b))
- Move lsp functionality to its own crate - ([23cec69](https://github.com/hadronomy/ram/commit/23cec690ba612cf79116c2825a152e64b9a6f854))
- Move ast and syntax_kind to the `ram_syntax` crate - ([1ed548b](https://github.com/hadronomy/ram/commit/1ed548ba482a79a06d7971137c143bbf75fdc332))
- Rename crates to follow `snake_case` convention - ([4c1e1cf](https://github.com/hadronomy/ram/commit/4c1e1cfed8402a3ccce3f3ed14c6ae59944e43ee))
- Split error module into its own crate - ([e3cf68d](https://github.com/hadronomy/ram/commit/e3cf68d39fa902281e1d87afe9caa3efac6d55b8))
- Divide `language` module into submodules - ([7213673](https://github.com/hadronomy/ram/commit/72136736fd41f8ed80a221ebae131dce2fec1f7c))
- Rename and cleanup modules - ([a1caad9](https://github.com/hadronomy/ram/commit/a1caad9229de88adc007fef24d1099a8e66314e3))
- Split parser into multiple - ([97f68e1](https://github.com/hadronomy/ram/commit/97f68e148897370f6b04c5e165e0e578b94410cf))

### 👷 CI/CD

- Configure basic cd workflow - ([08eb81b](https://github.com/hadronomy/ram/commit/08eb81b4fd7f4f52192b827f98a56c418c2d35db))

### 📚 Documentation

- Disable doctest for example ram program - ([e4bc124](https://github.com/hadronomy/ram/commit/e4bc124a8fffdddcc629eda5daad63925533755f))

### 📝 Other

- Merge branch 'main' into parser - ([4fdba73](https://github.com/hadronomy/ram/commit/4fdba733c99ac98d63b04958ff14aef292b0696e))

### 🎨 Styling

- Fix clippy warnings - ([018a526](https://github.com/hadronomy/ram/commit/018a52667c0d54056b5065e1d4f6a8282097f393))

### ⚙️ Miscellaneous Tasks

- Update all cargo dependencies - ([767830e](https://github.com/hadronomy/ram/commit/767830edc07c9ea3111874d4ef7dcea70223e49d))

