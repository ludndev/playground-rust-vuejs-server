# Playground Rust + Vue.js Server

A simple development server built with Rust that serves a Vue.js SPA (Single Page Application).

## Features

- Rust-based static file server
- SPA Web App support (here Vue.js but you can test other and make feedbacks)
- Custom web root directory support

## Project Structure

- `server/` - Rust backend server
- `web/` - Vue.js frontend application

## Installation

1. Clone the repository:
```bash
git clone https://github.com/ludndev/playground-rust-vuejs-server.git
```

2. Build and run the frontend:
```bash
cd web
npm ci
npm run build
```

3. Run the server:
```bash
cd server
cargo run
```

## Usage

- To run the server with a custom web root directory:
```bash
cd server
cargo run -- --dir path/to/web/root
```

If you are following this repository structure, no need to set `--dir` as default dir is set to `web/dist`.

### With Vue.js SPA

With Vue.js SPA, you can use the following command to run the server. Note that the `--dir` option is set to `../web/dist` for our demo Vue.js app. Feel free to adapt `--dir` to your actual path.

```bash
cargo run -- --dir ../web/dist
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
