# Blender Asset Tracer Codebase Map

This document provides a comprehensive overview of the `blender-asset-tracer` codebase, outlining the structure, definitions, and functions of each module.

## High-Level Overview

The `blender-asset-tracer` is a Python application designed to analyze and pack Blender `.blend` files and their dependencies. It consists of three main packages:

*   **`cli`**: Provides the command-line interface for the application.
*   **`blendfile`**: Handles the parsing and manipulation of `.blend` files.
*   **`trace`**: Implements the logic for tracing and identifying all external file dependencies.
*   **`pack`**: Contains the functionality for packing a `.blend` file and its dependencies into a single, portable package.

## `blender_asset_tracer.cli`

This package is responsible for the command-line interface. It uses `argparse` to define the available commands and their arguments.

### `__init__.py`

This file sets up the main parser and subparsers for the CLI, and it dispatches to the appropriate subcommand based on the user's input.

### `blocks.py`

The `blocks` command analyzes a `.blend` file and provides detailed information about the data blocks within it, including their size, count, and type. It can also dump the hex representation of the largest block for debugging purposes.

### `common.py`

This module provides a set of utility functions that are used across the different CLI subcommands. These include functions for adding flags to the argument parser, shortening file paths, and converting byte sizes into a human-readable format.

### `list_deps.py`

The `list` command is used to list all the external files (dependencies) that a `.blend` file uses. It can output the list in either a human-readable format or as JSON, and it has options to include SHA256 checksums and timing information for more detailed analysis.

### `pack.py`

The `pack` command is the most complex of the CLI commands. It is responsible for creating a "BAT-pack" of a `.blend` file and its dependencies. This can be a directory, a ZIP file, or even an upload to an S3 or Shaman server. It has several options for controlling the packing process, such as excluding files, compressing `.blend` files, and only packing assets with relative paths.

### `version.py`

The `version` command is a simple command that prints the current version of the `blender-asset-tracer` and exits.

## `blender_asset_tracer.blendfile`

This is the core package of the application. It's responsible for reading and parsing `.blend` files, including handling compressed files, decoding the DNA structure, and providing access to the data blocks.

### `__init__.py`

This file defines the `BlendFile` and `BlendFileBlock` classes, which are the main classes for working with `.blend` files. The `BlendFile` class represents a `.blend` file, and the `BlendFileBlock` class represents a single data block within the file. This file also contains the logic for caching `.blend` files to avoid re-parsing them unnecessarily.

### `dna_io.py`

This module provides the functionality for reading and writing the DNA structure of a `.blend` file. The DNA (Data-struct Name and Address) is a block in the `.blend` file that contains all the information about the data structures used in the file.

### `dna.py`

This module defines the classes that represent the different parts of the DNA structure, such as `Struct`, `Field`, and `Name`.

### `exceptions.py`

This module defines the custom exceptions that are used throughout the `blendfile` package.

### `header.py`

This module is responsible for reading and parsing the header of a `.blend` file. The header contains important information about the file, such as the Blender version it was created with, the pointer size, and the endianness.

### `iterators.py`

This module provides a set of iterators for traversing the data blocks in a `.blend` file.

### `magic_compression.py`

This module contains the logic for detecting and handling compressed `.blend` files. It can automatically decompress gzipped files into a temporary location so that they can be parsed by the rest of the `blendfile` package.

## `blender_asset_tracer.trace`

This package is responsible for finding all the dependencies of a `.blend` file.

### `__init__.py`

The `deps()` function in this file is the main entry point for the `trace` package. It uses a `BlockIterator` to iterate over all the blocks in the file, filters out the blocks that are known to not contain any external assets, and then calls `blocks2assets.iter_assets()` to find the actual dependencies.

### `blocks2assets.py`

This module contains the logic for finding the assets that are referenced by a given data block. It has a set of functions that know how to find the assets in different types of data blocks, such as materials, textures, and node trees.

### `expanders.py`

This module provides a set of functions for expanding file sequences, such as image sequences and UDIM textures.

### `file_sequence.py`

This module contains the logic for detecting and expanding file sequences.

### `file2blocks.py`

This module provides a `BlockIterator` class that can be used to iterate over all the data blocks in a `.blend` file.

### `modifier_walkers.py`

This module contains the logic for traversing the modifier stack of an object and finding any assets that are referenced by the modifiers.

### `progress.py`

This module defines a `Callback` class that can be used to report the progress of the tracing process.

### `result.py`

This module defines the `BlockUsage` class, which is used to represent a single usage of an asset by a data block.

## `blender_asset_tracer.pack`

This package is responsible for creating a BAT-pack.

### `__init__.py`

The `Packer` class in this file is the main class for the `pack` package. It has two main methods: `strategise()` and `execute()`. `strategise()` finds all the dependencies and determines what to do with them, and `execute()` performs the actual packing operation.

### `filesystem.py`

This module provides a `FileCopier` class that can be used to copy files from one location to another. It also has a `CompressedFileCopier` class that can be used to compress files on the fly.

### `progress.py`

This module defines a `Callback` class that can be used to report the progress of the packing process.

### `s3.py`

This module provides an `S3Packer` class that can be used to upload a BAT-pack to an S3-compatible storage service.

### `shaman.py`

This module provides a `ShamanPacker` class that can be used to upload a BAT-pack to a Shaman server.

### `transfer.py`

This module defines the `FileTransferer` class, which is responsible for transferring files from one location to another. It can be used to copy files to a local directory, a ZIP file, or a remote server.

### `zipped.py`

This module provides a `ZipPacker` class that can be used to create a ZIP archive of a BAT-pack.
