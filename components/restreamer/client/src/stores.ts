import { identity } from 'svelte/internal';
import { writable, get, type Writable } from 'svelte/store';
import { sanitizeLabel, sanitizeUrl } from './utils/util';

// Copied from 'svelte/store' as cannot be imported.
// See: https://github.com/sveltejs/svelte/pull/5887
/** Callback to inform of a value updates. */
declare type Subscriber<T> = (value: T) => void;
/** Unsubscribes from value updates. */
declare type Unsubscriber = () => void;
/** Callback to update a value. */
declare type Updater<T> = (value: T) => T;
/** Cleanup logic callback. */
declare type Invalidator<T> = (value?: T) => void;

enum Multiple {
  List = 'list',
  Json = 'json',
}

/**
 * State of the modal window for adding/editing re-streaming `Output`s.
 */
export class OutputModalState {
  /**
   * ID of the `Restream` to that the `Output` belongs to.
   */
  restream_id: string | null = null;

  /**
   * ID of the `Output` being edited in the [[`OutputModal`]] at the moment.
   *
   * If `null` then a new `Output` is being added.
   */
  edit_id: string | null = null;

  /**
   * Indicator whether the "Multiple list" or "Multiple json" tab is active in the [[`OutputModal`]].
   */
  multi: Multiple | false = false;

  /**
   * Check whether current tab is "Multiple list"
   */
  isMultiList = () => this.multi === Multiple.List;

  /**
   * Check whether current tab is "Multiple json"
   */
  isMultiJson = () => this.multi === Multiple.Json;

  /**
   * Label to be assigned to the `Output`.
   *
   * Empty string means no label.
   */
  label: string = '';

  /**
   * Url for preview of stream
   *
   */
  preview_url: string = '';

  /**
   * Previous value of `Output`'s preview URL before it has been edited in
   * the [[`OutputModal`]].
   *
   */
  prev_preview_url: string = '';

  /**
   * Previous label of the `Output` before it has been edited in the
   * [[`OutputModal`]].
   *
   * Empty string means no label.
   */
  prev_label: string | null = null;

  /**
   * Destination URL to re-stream a live stream onto with the `Output`.
   */
  url: string = '';

  /**
   * Previous value of `Output`'s destination URL before it has been edited in
   * the [[`OutputModal`]].
   */
  prev_url: string | null = null;

  /**
   * URLs to mix audio from with a live RTMP stream before outputting it.
   */
  mix_urls: string[] = [];

  /**
   * Previous value of `Output`'s mix URLs before they have been edited in the
   * [[`OutputModal`]].
   */
  prev_mix_urls: string[] | null = null;

  /**
   * List of multiple labels and RTMP URLs to be added in a comma-separated
   * format.
   */
  list: string = '';

  /**
   * Json representation of multiple outputs
   * format.
   */
  json: string = '';

  /**
   * Indicator whether the [[`OutputModal`]] is visible (opened) at the
   * moment.
   */
  visible: boolean = false;
}

/**
 * Shared reactive state of the modal window for adding restreaming `Output`s.
 */
export class OutputModal implements Writable<OutputModalState> {
  private state: Writable<OutputModalState> = writable(new OutputModalState());

  /** @inheritdoc */
  subscribe(
    run: Subscriber<OutputModalState>,
    invalidate?: Invalidator<OutputModalState>,
  ): Unsubscriber {
    return this.state.subscribe(run, invalidate);
  }

  /** @inheritdoc */
  set(v: OutputModalState) {
    v.url = sanitizeUrl(v.url);
    v.mix_urls = v.mix_urls.map(sanitizeUrl);
    this.state.set(v);
  }

  /** @inheritdoc */
  update(updater: Updater<OutputModalState>) {
    this.state.update(updater);
  }

  /**
   * Retrieves and returns current [[`OutputModalState`]].
   *
   * @returns    Current value of [[`OutputModalState`]].
   */
  get(): OutputModalState {
    return get(this.state);
  }

  /**
   * Opens this [[`OutputModal`]] window for adding a new `Ouput`.
   *
   * @param restream_id    ID of the `Restream` that a new `Ouput` being added
   *                       to.
   */
  openAdd(restream_id: string) {
    this.update((v) => {
      v.restream_id = restream_id;
      v.visible = true;
      return v;
    });
  }

  /**
   * Opens this [[`OutputModal`]] window for editing an existing `Ouput`.
   *
   * @param restream_id    ID of the `Restream` that an edited `Ouput` belongs
   *                       to.
   * @param id             ID of the `Output` being edited.
   * @param label          Current label of the `Output` before editing.
   * @param preview_url    Preview url for Output.
   * @param dst_url        Current destination URL of the `Output` before
   *                       editing.
   * @param mix_urls       Current mixing URLs of the `Output` before editing.
   */
  openEdit(
    restream_id: string,
    id: string,
    label: string | null,
    preview_url: string | null,
    dst_url: string,
    mix_urls: string[],
  ) {
    this.update((v) => {
      v.restream_id = restream_id;
      v.edit_id = id;

      v.prev_label = sanitizeLabel(label ?? '');
      v.label = v.prev_label;

      v.prev_preview_url = sanitizeUrl(preview_url ?? '');
      v.preview_url = v.prev_preview_url;

      v.prev_url = sanitizeUrl(dst_url);
      v.url = v.prev_url;

      v.prev_mix_urls = mix_urls.map(sanitizeUrl);
      v.mix_urls = v.prev_mix_urls.map(identity);

      v.multi = false;
      v.visible = true;
      return v;
    });
  }

  /**
   * Switches the current active tab of this [[`OutputModal`]] to "Single".
   */
  switchSingle() {
    this.update((v) => {
      v.multi = false;
      return v;
    });
  }

  /**
   * Switches the current active tab of this [[`OutputModal`]] to "Multiple.List".
   */
  switchMultiList() {
    this.update((v) => {
      v.multi = Multiple.List;
      return v;
    });
  }

  /**
   * Switches the current active tab of this [[`OutputModal`]] to "Multiple.List".
   */
  switchMultiJson() {
    this.update((v) => {
      v.multi = Multiple.Json;
      return v;
    });
  }

  /**
   * Removes the `i`-indexed mixin URL of this [[`OutputModal`]].
   */
  removeMixinSlot(i: number) {
    this.update((v) => {
      v.mix_urls.splice(i, 1);
      return v;
    });
  }

  /**
   * Adds a slot for a new mixin URL to this [[`OutputModal`]].
   */
  addMixinSlot() {
    this.update((v) => {
      v.mix_urls.push('');
      return v;
    });
  }

  /**
   * Sanitizes the current label value being input in this [[`OutputModal`]].
   */
  sanitizeLabel() {
    this.update((v) => {
      v.label = sanitizeLabel(v.label);
      return v;
    });
  }

  /**
   * Closes this [[`OutputModal`]] window.
   */
  close() {
    this.update((v) => {
      v.restream_id = null;
      v.edit_id = null;

      v.label = '';
      v.prev_label = null;

      v.preview_url = '';
      v.prev_preview_url = '';

      v.url = '';
      v.prev_url = null;

      v.mix_urls = [];
      v.prev_mix_urls = null;

      v.list = '';
      v.json = '';
      v.visible = false;
      return v;
    });
  }
}

/**
 * State of the modal window for adding exporting/importing `Inputs`s.
 */
export class ExportModalState {
  /**
   * ID of the `Restream` to operate on.
   *
   * If `null` then operates on all defined `Restream`s.
   */
  restream_id: string | null = null;

  /**
   * Current JSON value of the operated `Input`'s spec.
   */
  spec: string = '';

  /**
   * Previous JSON value of the operated `Input`'s spec.
   */
  prev_spec: string = '';

  /**
   * Indicator whether the [[`ExportModalModal`]] is visible (opened) at the
   * moment.
   */
  visible: boolean = false;
}

/**
 * Shared reactive state of the modal window for exporting/importing `Inputs`s.
 */
export class ExportModal implements Writable<ExportModalState> {
  private state: Writable<ExportModalState> = writable(new ExportModalState());

  /** @inheritdoc */
  subscribe(
    run: Subscriber<ExportModalState>,
    invalidate?: Invalidator<ExportModalState>,
  ): Unsubscriber {
    return this.state.subscribe(run, invalidate);
  }

  /** @inheritdoc */
  set(v: ExportModalState) {
    this.state.set(v);
  }

  /** @inheritdoc */
  update(updater: Updater<ExportModalState>) {
    this.state.update(updater);
  }

  /**
   * Retrieves and returns current [[`ExportModalState`]].
   *
   * @returns    Current value of [[`ExportModalState`]].
   */
  get(): ExportModalState {
    return get(this.state);
  }

  /**
   * Opens this [[`ExportModal`]] window for exporting/importing a `Restream`.
   *
   * @param id      ID of the `Restream` to be exported/imported.
   * @param spec    Current `Restream`'s spec received via GraphQL API.
   */
  async open(id: string | null, spec: string) {
    this.update((v) => {
      v.restream_id = id;
      v.spec = spec;
      v.prev_spec = spec;
      v.visible = true;
      return v;
    });
  }

  /**
   * Closes this [[`ExportModal`]] window.
   */
  close() {
    this.update((v) => {
      v.restream_id = null;
      v.spec = '';
      v.prev_spec = '';
      v.visible = false;
      return v;
    });
  }
}

/**
 * Global singleton instance of an [[`OutputModal`]] window used by this
 * application.
 */
export const outputModal = new OutputModal();

/**
 * Global singleton instance of an [[`ExportModal`]] window used by this
 * application.
 */
export const exportModal = new ExportModal();
