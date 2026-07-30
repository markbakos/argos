# Argos

Argos is a local-first, modular control center for systemd-based Linux desktops. Its first target is a single-user Arch Linux and GNOME workstation.

The repository is in the **foundation implementation phase**. The foundation documentation was approved for implementation by the user on 2026-07-30, and work follows the dependency-ordered foundation task ledger.

Start with the [documentation index](docs/README.md), then read the [product definition](docs/product.md), [architecture](docs/architecture.md), and [foundation plan](docs/foundation/README.md).

## Current phase

- Product, architecture, safety boundaries, and foundation specifications: approved
- Expected behavior and implementation tasks: approved
- Foundation implementation: `FND-BST-001` and `FND-BST-002` complete; `FND-BST-003` next

The implementation order and completion gates are documented in [Foundation planning](docs/foundation/README.md) and the [task ledger](docs/foundation/tasks.md).

## Prerequisites

Development is currently tested on Arch Linux with GNOME/systemd. Install the native build dependencies:

```sh
sudo pacman -S --needed base-devel corepack curl file git nodejs npm openssl sqlite webkit2gtk-4.1 wget
```

You also need:

- Rust 1.97.1 managed by `rustup`, including Clippy and rustfmt;
- Node.js 24.15 or newer (Node.js 24 LTS is the CI target);
- pnpm 11.18.0 through Corepack.

See the [development guide](docs/development.md#tested-bootstrap-baseline) for the tested toolchain and dependency rationale.

## Set up the repository

From the repository root, install the pinned toolchains and dependencies:

```sh
rustup toolchain install 1.97.1 --profile minimal --component clippy --component rustfmt
sudo corepack enable pnpm
pnpm --version
pnpm install --frozen-lockfile
```

The version check should print `11.18.0`; Corepack selects it from the root `packageManager` field. Keep the committed Cargo and pnpm lockfiles unchanged during setup.

## Run Argos

Start the Tauri development application:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 pnpm dev
```

This launches the Vite frontend and opens the Argos desktop window. Source builds embed the `development` runtime profile; they do not select the production profile. The current foundation scaffold displays a minimal loading window while later tasks implement the application features. Close the main window to exit Argos.

To verify the workspace without launching the desktop window:

```sh
pnpm check
```

Other useful commands:

| Command            | Purpose                                                            |
| ------------------ | ------------------------------------------------------------------ |
| `pnpm build`       | Build the frontend and Rust workspace without packaging            |
| `pnpm tauri:build` | Build an optimized, non-bundled development-profile desktop binary |
| `pnpm test`        | Run Rust, frontend, and boundary tests                             |
| `pnpm lint`        | Run ESLint and Clippy with warnings denied                         |
| `pnpm format`      | Format TypeScript, CSS, Markdown, and Rust sources                 |

Packaging and production-profile builds are not implemented yet. The complete command contract is in the [development guide](docs/development.md#command-surface).
