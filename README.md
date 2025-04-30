<div align="center">
  <img src="/.github/images/github-header-image.webp" alt="GitHub Header Image" width="auto" />

  <!-- MIT License -->
  <a href="https://github.com/hadronomy/ram/blob/main/LICENSE">
    <img
      alt="Content License"
      src="https://img.shields.io/github/license/hadronomy/ram?style=for-the-badge&logo=starship&color=ee999f&logoColor=D9E0EE&labelColor=302D41"
    />
  </a>

  <!-- GitHub Repo Stars -->
  <a href="https://github.com/hadronomy/ram/stargazers">
    <img
      alt="Stars"
      src="https://img.shields.io/github/stars/hadronomy/ram?style=for-the-badge&logo=starship&color=c69ff5&logoColor=D9E0EE&labelColor=302D41"
    />
  </a>
  <p></p>
  <span>
    <code>RAM</code> <i>(Random Access Machine)</i> <strong>language</strong> and <strong>emulator</strong>.
  </span>
  <p></p>
  <!-- <a href="#installation">Installation</a> • -->
  <a href="#requirements">Requirements</a> •
  <a href="#license">License</a>
  <hr />

</div>

## About This Project

This project aims to develop a complete ecosystem for the Random Access Machine (RAM) model of computation. `RAM` provides both a formally defined **language** based on this model and a robust **emulator** to execute programs written in that language.

### Objectives

*   **Formal Language Definition:** Define a clear, consistent, and unambiguous specification for the RAM programming language.
*   **Accurate Emulation:** Implement an emulator that faithfully executes RAM programs according to the defined semantics.
*   **Performance:** Strive for reasonable performance in the emulator to handle non-trivial programs.
*   **Educational Resource:** Serve as a practical tool and codebase for learning about the RAM model, theoretical computer science concepts, and potentially compiler/interpreter design.
*   **(Future) LLVM Backend:** Explore replacing the emulator with an LLVM-based backend for potentially higher performance code generation.
*   **(Future) Extensibility:** Design the core components with potential future extensions in mind, such as debugging tools, visualization, or integration with other systems.

> [!CAUTION]
> This project is under heavy development
> It's not in any way stable or even functional in some cases.
> **Until the first release** the `main` branch is not guaranteed
> to work in any way.

## Requirements

- mise - [Installation](https://mise.jdx.dev)
- mold - [Installation](https://github.com/rui314/mold)
- rustup/cargo - [Installation](https://rustup.rs)

`mise` Is used to manage the project dependencies and environment.
To ensure that you have the needed tools installed, run the following command:

```bash
mise trust
mise install
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) for details.
