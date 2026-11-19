// The capacity tab: what the daemon could still use on a provider, as the
// provider last said, following it as it changes.

import type { Capacity } from "../world/model";
import { bytes } from "./strip";

/** The four amounts, in the order they are shown: the member of the provider's answer, the row's name, and what the provider must offer for the amount to mean anything. */
const ROWS = [
  { member: "volumes_disk", name: "Volumes", offer: "volumes" },
  { member: "images_disk", name: "Images", offer: "images" },
  { member: "container_memory", name: "Container memory", offer: "containers" },
  { member: "overlay_disk", name: "Overlay disk", offer: "overlay" },
] as const;

/**
 * The capacity tab: shown from the moment it is made until `dispose`.
 *
 * The rules:
 *
 * 1. It asks nobody. What it shows is the provider's `capacity` in the
 *    board's own `providers list`, which the daemon asks the provider
 *    for every ten seconds, and at once after anything it does that
 *    changes it; `set` is handed each one the list brings.
 * 2. Four rows, a name and an amount: volumes, images, container
 *    memory, overlay disk — the one room a container's own disk, an
 *    ephemeral volume it mounts and an ephemeral volume's FUSE serve
 *    are all set aside from. Each amount is what the daemon could
 *    still use of it now, in words (`bytes`). The provider says what
 *    is free and never a total, so no row draws a share of anything.
 * 3. A row whose resource the provider does not offer says so in
 *    place of its amount: the provider sends a number for it all the
 *    same, and the number means nothing.
 * 4. A provider that is not connected, and one connected that has not
 *    answered yet, each say so in place of the rows.
 * 5. `set` writes only the words that differ from what is shown, so an
 *    amount that did not change is not touched.
 * 6. `dispose` — the sheet closed, or swapped to another node — empties
 *    the pane.
 */
export class CapacityPane {
  private readonly rows: HTMLElement;
  private readonly amounts: HTMLElement[];
  private readonly none: HTMLElement;

  constructor(
    private readonly pane: HTMLElement,
    capacity: Capacity,
  ) {
    const note = document.createElement("p");
    note.className = "note";
    note.textContent = "Available to this daemon now";
    // Rule 2.
    this.rows = document.createElement("dl");
    this.amounts = ROWS.map((row) => {
      const name = document.createElement("dt");
      name.textContent = row.name;
      const amount = document.createElement("dd");
      this.rows.append(name, amount);
      return amount;
    });
    // Rule 4.
    this.none = document.createElement("p");
    this.none.className = "none";
    this.pane.append(note, this.rows, this.none);
    this.set(capacity);
  }

  /** Rules 3 to 5: the provider's capacity, as the list has it now. */
  set(capacity: Capacity): void {
    const amounts = capacity.connected ? capacity.amounts : undefined;
    this.rows.hidden = !amounts;
    this.none.hidden = !!amounts;
    if (!amounts) {
      write(this.none, capacity.connected ? "Waiting for the provider's first answer" : "Not connected");
      return;
    }
    ROWS.forEach((row, index) => {
      const amount = this.amounts[index];
      if (!amount) return;
      const offered = capacity.offers[row.offer];
      amount.classList.toggle("off", !offered);
      write(amount, offered ? bytes(amounts[row.member]) : "Not offered");
    });
  }

  /** Rule 6. */
  dispose(): void {
    this.pane.replaceChildren();
  }
}

/** Rule 5: `words` in `element`, where they are not there already. */
function write(element: HTMLElement, words: string): void {
  if (element.textContent !== words) element.textContent = words;
}
