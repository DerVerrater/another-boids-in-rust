# Another Boids in Rust

This is my implementation of the Boids flocking algorithm. It is written in Rust and built on the Bevy game engine.

## Building & Running

### Desktop/native

You'll need a working Rust toolchain, of course. See the [rustup](https://rustup.rs/) site for the basics.

```sh
# Install system dependencies

:~$ sudo apt install libasound2-dev libudev-dev pkg-config # Debian 13
:~$ sudo yum install alsa-lib-devel libgudev-devel ppkgconf-pkg-config # Fedora 42

# Build and run
:~$ cargo build --release
:~$ cargo run --release # Or skip the build and only run. Cargo will (re)build as-needed.
```

### Web

This project creates a "static site," meaning a complete deployment of the site is simply copying the output folder onto a webserver. There are **two** ways to use this.

1. A standalone application which can be quickly hosted as-is. This includes the WASM, it's JS glue, and an index.html page.
2. A sub-page in a larger website. This is actually the same, but names it's HTML page "boids.html" so consumers (you) can provide their own index.html.
    - I'm not using a JS Bundler at this time. If you're familiar with JS development, this probably looks like a dumb way to do it. Sorry about that.

#### Standalone build:

1. Build `make web-standalone`
2. Serve `python3 -m http.server -d ./out`
3. Visit site in browser: `http://localhost:8000`

To quickly get a tarball, use `make tarball_standalone`. If you're trying to build and upload the program somewhere, this may provide a bit of convenience. Compressing it may be a good idea, too.

#### "Bundle-able" build

For a "bundle-able" build, you'll need to write your own index.html and link to the boids.html file.

Basically, just throw in a hyperlink with `<a href="boids.html">Boids</a>`

```html
<!DOCTYPE html>
<html>
    <!-- your page content, etc -->
    <body>
        <a href="boids.html">Boids</a>
    </body>
    <!-- more page content, etc -->
</html>
```

As mentioned in the option 2 description, I'm not using a JS Bundler. There is no "package.json" or anything to integrate properly with a JS framework. I plan to fix that at some point, but for now there are just a bunch of files to grab.

---

You can use any HTTP server you like. In the steps above, I'm using the Python3 built-in [http.server module](https://docs.python.org/3/library/http.server.html); which is **NOT** recommended for production use. Don't put that on the Internet! Alternatives include [Miniserve](https://crates.io/crates/miniserve) and [BusyBox](https://busybox.net/). The latter of which I'm using in the Docker image.

### Web, but as a Docker Container

You'll need a working Docker installation. See the [get-docker](https://docs.docker.com/get-started/get-docker/) page for details. The rest of the build and execution is self-contained in the container(s) and will not require additional host tools.

```sh
:~$ docker build -t boids-website .
:~$ docker run -it --rm boids-website
```

You might have to open another terminal to `docker kill <the boids container>`. It doesn't seem to respond to `ctrl+c` and I don't plan to figure out why any time soon.

You may also notice that the Dockerfile doesn't call on the Makefile. This is because the Dockerfile predates the Makefile and I haven't updated it to follow the manual process described above. The result is the same, only now it's in a container and using the BusyBox webserver.

## Controls

| Input | Effect |
|-|-|
| Mouse | The scanner circle is attached to the mouse cursor. Move it to scan boids within the radius. |
| Left mouse button | Put scanner into center-of-mass mode |
| Right mouse button | Put scanner into average velocity mode |

**BUG:** On web builds, the right-click action will *both* switch the scanner mode and open the browser's context menu. You can work around this by closing the menu without left-clicking on the canvas -- click outside the canvas, or use the escape key, etc.
