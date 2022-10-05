# nail
> A fast static blog engine.

## Installation

```
cargo install nail-blog
```

## Usage

```sh
nail new my-blog # create blog
cd my-blog # enter blog directory
nail post new "Hello World" # create new post
nail dev # serve blog locally
nail build # build blog for production
```

## Roadmap

- [ ] Blog Management
  - [x] Create new blog
  - [x] Create new post
  - [ ] Publish/unpublish post
- [x] Building
  - [x] Build blog
  - [x] Incremental builds
- [ ] Local Development
  - [x] Serve blog locally
  - [x] Rebuild on change
  - [ ] Live reload