<script lang="js">
  import { onDestroy } from 'svelte';
  import { mutation } from 'svelte-apollo';
  import { SetOutput } from '../../api/client.graphql';
  import { outputModal as value } from '../stores';
  import { sanitizeLabel, sanitizeUrl, showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const setOutputMutation = mutation(SetOutput);

  let submitable = false;
  let invalidLine;
  let invalidJson;

  onDestroy(
    value.subscribe((v) => {
      invalidLine = '';
      invalidJson = '';

      if (v.multi) {
        submitable =
          (v.isMultiList() && v.list !== '' && !invalidLine) ||
          (v.isMultiJson() && v.json !== '' && !invalidJson);
      } else {
        submitable = v.url !== '';
        let changed = !v.edit_id;
        if (!!v.edit_id) {
          changed |=
            v.label !== v.prev_label ||
            v.url !== v.prev_url ||
            v.preview_url !== v.prev_preview_url ||
            JSON.stringify(v.mix_urls) !== JSON.stringify(v.prev_mix_urls);
        }
        if (v.mix_urls.length > 0) {
          submitable &= v.mix_urls.every((m) => m !== '');
        }
        submitable &= changed;
      }
    })
  );

  /**
   * Sanitizes the given `list` of multiple labels and URLs.
   *
   * @param list string    List of comma-separated labels and URLs to sanitize.
   *
   * @returns string    Sanitized list.
   */
  function sanitizeList(list) {
    if (list === '') return list;
    return list
      .trim()
      .split(/\r\n|\r|\n/)
      .map((line) => {
        const p = line.trim().split(',');
        let i = 0;
        if (p.length > 1) {
          p[i] = sanitizeLabel(p[i]);
          i += 1;
        }
        for (; i < p.length; i += 1) {
          p[i] = sanitizeUrl(p[i]);
        }
        return p.filter((v) => v !== '').join(',');
      })
      .filter((line) => line !== '')
      .join('\n');
  }

  // Wraps list of objects into square brackets.
  // This is used when we paste formatted list of objects into multi-json tab
  function fixJsonArray() {
    let json = $value.json;
    const len = $value.json.length;
    json.trim();

    const isArrayOfObjects =
      json && json.charAt(0) === '{' && json.charAt(len - 1) === '}';
    if (isArrayOfObjects) {
      json = json
        .trim()
        .split(/\r\n|\r|\n/)
        .join(',');
      $value.json = `[${json}]`;
    }
  }

  function revalidateJson() {
    const v = value.get();
    invalidJson = '';
    if (v.json.trim()) {
      try {
        JSON.parse(v.json);
      } catch (e) {
        invalidJson =
          'Failed to parse JSON: ' + e.message + '. Please follow the example:';
      }
    }
  }

  function revalidateList() {
    value.update((v) => {
      v.list = sanitizeList(v.list);

      const lines = v.list.split(/\r\n|\r|\n/);
      const invalidIndex = lines.findIndex(
        (line) => line.split(',').length > 2
      );
      invalidLine =
        invalidIndex !== -1
          ? invalidIndex + 1 + ': ' + lines[invalidIndex]
          : undefined;

      return v;
    });
  }

  async function submit() {
    let v = value.get();
    if (v.multi) {
      if (v.isMultiList()) {
        revalidateList();
      } else if (v.isMultiJson()) {
        revalidateJson();
      } else {
        throw new Error('Unknown list type');
      }
    }

    if (!submitable) return;

    let submit = [];
    v = value.get();
    if (v.multi) {
      if (v.isMultiList()) {
        v.list.split(/\r\n|\r|\n/).forEach((line) => {
          const vs = line.split(',');
          let vars = {
            restream_id: v.restream_id,
            url: vs[vs.length - 1],
            mixins: [],
          };
          if (vs.length > 1) {
            vars.label = vs[0];
          }
          submit.push(vars);
        });
      } else if (v.isMultiJson()) {
        //
        try {
          submit = JSON.parse(v.json.trim()).map((x) => ({
            restream_id: v.restream_id,
            url: sanitizeUrl(x.url),
            mixins: [],
            ...(x.label && { label: sanitizeLabel(x.label) }),
            ...(x.preview_url && { preview_url: sanitizeUrl(x.preview_url) }),
          }));
        } catch (e) {
          showError('Failed to add ' + variables.url + ':\n' + e.message);
          failed.push(variables);
          return;
        }
      }
    } else {
      let vars = {
        restream_id: v.restream_id,
        url: sanitizeUrl(v.url),
        mixins: [],
      };
      const label = sanitizeLabel(v.label);
      if (label !== '') {
        vars.label = label;
      }

      const preview_url = sanitizeUrl(v.preview_url);
      if (preview_url !== '') {
        vars.preview_url = preview_url;
      }

      if (v.mix_urls.length > 0) {
        vars.mixins = v.mix_urls;
      }
      if (v.edit_id) {
        vars.id = v.edit_id;
      }
      submit.push(vars);
    }
    if (submit.length < 1) return;

    let failed = [];
    await Promise.all(
      submit.map(async (variables) => {
        try {
          await setOutputMutation({ variables });
        } catch (e) {
          showError('Failed to add ' + variables.url + ':\n' + e.message);
          failed.push(variables);
        }
      })
    );

    if (failed.length < 1) {
      value.close();
      return;
    }

    value.update((v) => {
      if (v.isMultiList()) {
        v.list = failed
          .map((vars) => {
            return (vars.label ? vars.label + ',' : '') + vars.url;
          })
          .join('\n');
      } else if (v.isMultiJson()) {
        v.json = JSON.stringify(
          failed.map((x) => {
            const { url, label, preview_url } = x;
            return { url, label, preview_url };
          })
        );
      }

      return v;
    });
  }

  function addMixinSlot(event) {
    value.addMixinSlot();
    event.currentTarget.checked = false;
  }

  function removeMixinSlot(event, i) {
    value.removeMixinSlot(i);
    event.currentTarget.checked = true;
  }

  function close() {
    value.close();
  }

  const multipleNoteTemplate = `
      <div class="uk-alert">
        Server will publish the input live stream to these addresses.
        <br />
        Supported protocols: <code>rtmp://</code>, <code>icecast://</code>
    </div>
`;

  const multiListPlaceholderText = `One line - one address (with optional label):
  label1,rtmp://1...
  rtmp://2...
  label3,rtmp://3..."
`;

  const multiJsonPlaceholderText = `Array of outputs (Fields: 'label' and 'preview_url' are optional) :
[
  { "label": "label1", "url": "rtmp://1...", "preview_url": "https://1..." },
  { "label": "label2", "url": "rtmp://2..." },
  { "url": "rtmp://3..." }
]
`;
</script>

<template>
  {#if $value.visible}
    <div
      class="uk-modal uk-open"
      use:saveOrCloseByKeys={{ save: submit, close: close }}
    >
      <div
        class="uk-modal-dialog uk-modal-body"
        class:is-multi-list={$value.isMultiList()}
        class:is-multi-json={$value.isMultiJson()}
      >
        <h2 class="uk-modal-title">
          {#if !$value.edit_id}
            Add new output destination{$value.multi ? 's' : ''} for re-streaming
          {:else}
            Edit output destination for re-streaming
          {/if}
        </h2>
        <button
          class="uk-modal-close-outside"
          uk-close
          type="button"
          on:click={close}
        />

        {#if !$value.edit_id}
          <ul class="uk-tab">
            <li class:uk-active={!$value.multi}>
              <a href="/" on:click|preventDefault={() => value.switchSingle()}
                >Single</a
              >
            </li>
            <li class:uk-active={$value.isMultiList()}>
              <a
                href="/"
                on:click|preventDefault={() => value.switchMultiList()}
                >Multiple - list</a
              >
            </li>
            <li class:uk-active={$value.isMultiJson()}>
              <a
                href="/"
                on:click|preventDefault={() => value.switchMultiJson()}
                >Multiple - Json</a
              >
            </li>
          </ul>
        {/if}

        <fieldset class="single-form">
          <input
            class="uk-input uk-form-small"
            type="text"
            data-testid="add-output-modal:label-input"
            bind:value={$value.label}
            on:change={() => value.sanitizeLabel()}
            placeholder="optional label"
          />
          <input
            class="uk-input"
            type="text"
            data-testid="add-output-modal:rtmp-input"
            bind:value={$value.url}
            placeholder="rtmp://..."
          />
          <input
            class="uk-input"
            type="text"
            data-testid="add-output-modal:preview-input"
            bind:value={$value.preview_url}
            placeholder="optional preview url"
          />
          <div class="uk-alert">
            Server will publish the input live stream to this address.
            <br />
            Supported protocols:
            <code>rtmp://</code>,
            <code>srt://</code>,
            <code>icecast://</code>,
            <code>file:///.flv</code>
          </div>

          {#each $value.mix_urls as mix_url, i}
            <label class="mix-with">
              <input
                class="uk-checkbox"
                type="checkbox"
                checked
                on:change={(ev) => removeMixinSlot(ev, i)}
              /> mix with</label
            >
            <input
              class="uk-input"
              type="text"
              bind:value={mix_url}
              placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>&identity=<identity>"
            />
          {/each}

          {#if $value.mix_urls.length < 5}
            <label class="mix-with">
              <input
                class="uk-checkbox"
                type="checkbox"
                on:change={addMixinSlot}
              /> mix with</label
            >
          {/if}

          {#if $value.mix_urls.length > 0}
            <div class="uk-alert">
              Server will mix the input live stream with the address{$value
                .mix_urls.length > 1
                ? 'es'
                : ''} above.
              <br />
              Supported protocols: <code>ts://</code>, <code>http://.mp3</code>
              <br /><br />
              For <code>ts://</code>:
              <br />
              If <code>name</code> is not specified than the label value will be
              used, if any, or a random generated one.
              <br />
              If <code title="Should be escaped or it won't work">identity</code> is not specified than the a random generated
              one.
            </div>
          {/if}
        </fieldset>

        <fieldset class="multi-list-form">
          {#if !!invalidLine}
            <span class="uk-form-danger line-err"
              >Invalid line {invalidLine}</span
            >
          {/if}
          <textarea
            class="uk-textarea"
            class:uk-form-danger={!!invalidLine}
            bind:value={$value.list}
            on:change={revalidateList}
            placeholder={multiListPlaceholderText}
          />
          {@html multipleNoteTemplate}
        </fieldset>

        <fieldset class="multi-json-form">
          <textarea
            class="uk-textarea"
            class:uk-form-danger={!!invalidJson}
            bind:value={$value.json}
            on:change={revalidateJson}
            on:input={fixJsonArray}
            placeholder={multiJsonPlaceholderText}
          />
          {#if !!invalidJson}
            <div class="uk-form-danger json-err">
              <span class="">{invalidJson}</span>
              <pre>
              <code>
                {multiJsonPlaceholderText}
              </code>
          </pre>
            </div>
          {/if}

          {@html multipleNoteTemplate}
        </fieldset>

        <button
          data-testid="add-output-modal:confirm"
          class="uk-button uk-button-primary"
          disabled={!submitable}
          on:click={submit}>{!$value.edit_id ? 'Add' : 'Edit'}</button
        >
      </div>
    </div>
  {/if}
</template>

<style lang="stylus">
  .uk-modal
    &.uk-open
      display: block

    .uk-modal-title
      font-size: 1.5rem

    fieldset
      border: none
      padding: 0

      > input
        margin-bottom: 5px

      .uk-form-small
        width: auto

      .uk-textarea
        min-height: 200px
        resize: none

      .uk-alert
        font-size: 14px
        margin-bottom: 0

      .line-err
        display: block
        font-size: 11px
      .json-err
        font-size: 11px
        pre
          font-size: inherit

    .single-form
      display: block

    .multi-list-form,.multi-json-form
      display: none

    .is-multi-list,.is-multi-json
      .single-form
        display: none

    .is-multi-list
      .multi-list-form
        display: block

    .is-multi-json
      .multi-json-form
        display: block

    .mix-with
      display: block
      margin-top: 16px
</style>
