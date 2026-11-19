// The page: one Phaser game filling the window. The backdrop scene
// draws the endless grid beneath; the board scene draws every entity
// over it; the UI scene takes the keys. The outliner floats over the
// board in HTML, and the tab strip along the top adds a board per daemon
// opened, and switches among them. The daemon is reached through
// `lib/wire.ts` once there is something to show. (The hex map's scenes,
// `Sea` and `World`, are kept beside them.) The game draws in the screen's
// own pixels, so the board is sharp on any screen (`lib/density.ts`).

import "../../diverge-theme/theme.css";
import "./main.css";
import * as Phaser from "phaser";
import { density, onDensity, watchDensity } from "./lib/density";
import { int } from "./lib/theme";
import { outliner } from "./outliner/outliner";
import { Backdrop } from "./scenes/backdrop";
import { Board } from "./scenes/board";
import { Ui } from "./scenes/ui";
import { tabs } from "./tabs/tabs";

/** The keys that move among things on the page: while one is used, hover marks nothing. */
const MOVING = new Set(["Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"]);

/**
 * Which hand is in use, on the root as `data-input`: `keys` from a key
 * that moves among things until the mouse moves, `mouse` otherwise. The
 * page's styles mark hover only for the mouse, so the keys' focus and
 * the mouse's hover never mark two things at once.
 */
function hands(): void {
  const root = document.documentElement;
  root.dataset.input = "mouse";
  addEventListener("keydown", (event) => MOVING.has(event.key) && (root.dataset.input = "keys"), true);
  addEventListener("pointermove", () => root.dataset.input !== "mouse" && (root.dataset.input = "mouse"), true);
}

hands();
const world = document.getElementById("world") as HTMLElement;
/** The window's size in the screen's own pixels. */
const size = () => ({ width: Math.round(world.clientWidth * density()), height: Math.round(world.clientHeight * density()) });
/**
 * How the game is sized. Two sizes, each owned by one side.
 *
 * 1. **How big the canvas is shown is the page's.** The canvas fills
 *    `#world` by a rule of `main.css`, as any element fills its box:
 *    nothing here sets it, so a window resized, maximised, restored
 *    or snapped is followed by the page laying itself out, with no
 *    code run. The game is told no zoom, so the library writes no
 *    size on the canvas of its own.
 * 2. **How many pixels it draws is the game's.** A canvas does not
 *    change its own resolution: `fit` gives it `#world`'s size in the
 *    screen's own pixels, so one of its pixels is one of the screen's
 *    and nothing it draws is stretched. It is told by an observer of
 *    `#world`, after the page is laid out, and when the screen's
 *    density changes.
 * 3. Its smaller copies of each texture (mipmaps) keep the nodes'
 *    shapes steady zoomed out.
 */
const game = new Phaser.Game({
  type: Phaser.AUTO,
  parent: world,
  backgroundColor: int("bg"),
  scale: { mode: Phaser.Scale.NONE, ...size() },
  render: { mipmapFilter: "LINEAR_MIPMAP_LINEAR" },
  scene: [Backdrop, Board, Ui],
});
/** Rule 2: the game's resolution, as `#world` is now. A fit to the size it already is does nothing. */
const fit = () => {
  const { width, height } = size();
  if (width === 0 || height === 0) return;
  if (game.scale.width === width && game.scale.height === height) return;
  game.scale.resize(width, height);
};
new ResizeObserver(fit).observe(world);
watchDensity();
onDensity(fit);
tabs(game);
outliner(game);
