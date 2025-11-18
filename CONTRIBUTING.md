# Contributing to baremetal-win11

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

1. **Prerequisites**:
   - Windows 10/11
   - Rust 1.70 or later
   - Visual Studio Build Tools (for windows-sys)

2. **Clone and Build**:
   ```bash
   git clone https://github.com/Napolitain/baremetal-win11.git
   cd baremetal-win11
   cargo build
   ```

3. **Run Tests**:
   ```bash
   cargo test
   cargo clippy
   ```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Ensure no Clippy warnings: `cargo clippy`
- Use meaningful variable and function names
- Add comments for complex logic only

## Adding New Process Patterns

To add support for new applications:

1. **Identify the category**:
   - Critical: System processes (never add here without careful consideration)
   - Gaming: Games and game-related services
   - Communication: Chat, voice, video apps
   - BackgroundService: Launchers, updaters, utilities
   - Productivity: Browsers, office apps, IDEs

2. **Add pattern to categorize_process()**:
   ```rust
   // In the appropriate category section
   let your_category_patterns = [
       "yourapp.exe",
       "yourapp-helper.exe",
   ];
   ```

3. **Test with the actual application**:
   ```bash
   cargo run --release -- --verbose --all
   ```

4. **Update documentation**:
   - Add to CATEGORIZATION_STRATEGIES.md
   - Update README.md if it's a notable addition

## Submitting Changes

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write clean, documented code
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`

3. **Commit with clear messages**:
   ```bash
   git commit -m "Add support for XYZ application"
   ```

4. **Push and create a Pull Request**:
   ```bash
   git push origin feature/your-feature-name
   ```

## Pull Request Guidelines

- **Title**: Clear, descriptive title
- **Description**: 
  - What changes were made
  - Why these changes are needed
  - How to test the changes
- **Testing**: Describe how you tested the changes
- **Documentation**: Update relevant documentation

## Reporting Issues

When reporting bugs or requesting features:

1. **Check existing issues** first
2. **Provide details**:
   - Windows version
   - Rust version
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - Relevant logs or screenshots

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## Questions?

Feel free to open an issue for questions or discussion!
