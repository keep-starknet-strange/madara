# papyrus-config

## Description

papyrus-config is a flexible and powerful layered configuration system designed
specifically for Papyrus, a Starknet node. This system allows you to easily
manage configurations for your Papyrus node by leveraging various sources and
providing additional helpful features.

## Configuration sources

Supports multiple configuration sources in ascending order of overriding
priority:

- Default values
- Configuration files (from first to last)
- Environment variables
- Command-line arguments

## Additional features

- **Support for Nested Configuration Components:** Organize your configurations
  into nested components, making it easy to manage complex settings for
  different aspects of the application.

- **Usage of Pointers:** Use pointers to merge parameters that are common to
  multiple components. This capability helps in streamlining configurations and
  avoiding duplication of settings.

- **Automatically-Generated Command Line Parser:** To simplify the process of
  handling command-line arguments, the system automatically generates a
  command-line parser. This means you don't have to write complex argument
  parsing code; it's ready to use out-of-the-box.

- **Automatically-Generated Reference Configuration File:** Makes it easier for
  users by generating a reference configuration file. This file serves as a
  template that highlights all available configuration options and their default
  values, enabling users to customize their configurations efficiently.
