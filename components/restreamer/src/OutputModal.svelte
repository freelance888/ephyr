<script lang="js">
  import { onDestroy } from 'svelte';
  import { mutation } from 'svelte-apollo';

  import { SetOutput } from './api/graphql/client.graphql';

  import { outputModal as value } from './stores';

  import { sanitizeLabel, sanitizeUrl, showError } from './util';

  const setOutputMutation = mutation(SetOutput);

  let submitable = false;
  let invalidLine;
  onDestroy(
    value.subscribe((v) => {
      if (v.multi) {
        submitable = v.list !== '' && !invalidLine;
      } else {
        submitable = v.url !== '';
        let changed = !v.edit_id;
        if (!!v.edit_id) {
          changed |=
            v.label !== v.prev_label ||
            v.url !== v.prev_url ||
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
    revalidateList();
    if (!submitable) return;

    let submit = [];
    const v = value.get();
    if (v.multi) {
      v.list.split(/\r\n|\r|\n/).forEach((line) => {
        const vs = line.split(',');
        let vars = {
          restream_id: v.restream_id,
          url: vs[vs.length - 1],
        };
        if (vs.length > 1) {
          vars.label = vs[0];
        }
        submit.push(vars);
      });
    } else {
      let vars = {
        restream_id: v.restream_id,
        url: sanitizeUrl(v.url),
      };
      const label = sanitizeLabel(v.label);
      if (label !== '') {
        vars.label = label;
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
      v.list = failed
        .map((vars) => {
          return (vars.label ? vars.label + ',' : '') + vars.url;
        })
        .join('\n');
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
</script>

<template>
  <div class="uk-modal" class:uk-open={$value.visible}>
    <div class="uk-modal-dialog uk-modal-body" class:is-multi={$value.multi}>
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
        on:click={() => value.close()}
      />

      {#if !$value.edit_id}
        <ul class="uk-tab">
          <li class:uk-active={!$value.multi}>
            <a href="/" on:click|preventDefault={() => value.switchSingle()}
              >Single</a
            >
          </li>
          <li class:uk-active={$value.multi}>
            <a href="/" on:click|preventDefault={() => value.switchMulti()}
              >Multiple</a
            >
          </li>
        </ul>
      {/if}

      <fieldset class="single-form">
        <input
          class="uk-input uk-form-small"
          type="text"
          bind:value={$value.label}
          on:change={() => value.sanitizeLabel()}
          placeholder="optional label"
        />
        <input
          class="uk-input"
          type="text"
          bind:value={$value.url}
          placeholder="rtmp://..."
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
            placeholder="ts://<teamspeak-host>:<port>/<channel>?name=<name>"
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
            If <code>name</code> is not specified than the label value will be used,
            if any, or a random generated one.
          </div>
        {/if}
      </fieldset>

      <fieldset class="multi-form">
        {#if !!invalidLine}
          <span class="uk-form-danger line-err">Invalid line {invalidLine}</span
          >
        {/if}
        <textarea
          class="uk-textarea"
          class:uk-form-danger={!!invalidLine}
          bind:value={$value.list}
          on:change={revalidateList}
          placeholder="One line - one address (with optional label):
label1,rtmp://1...
rtmp://2...
label3,rtmp://3..."
        />
        <div class="uk-alert">
          Server will publish the input live stream to these addresses.
          <br />
          Supported protocols: <code>rtmp://</code>, <code>icecast://</code>
        </div>
      </fieldset>

      <button
        class="uk-button uk-button-primary"
        disabled={!submitable}
        on:click={submit}>{!$value.edit_id ? 'Add' : 'Edit'}</button
      >
    </div>
  </div>
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

      .uk-form-small
        width: auto
        margin-bottom: 5px

      .uk-textarea
        min-height: 200px
        resize: none

      .uk-alert
        font-size: 14px
        margin-bottom: 0

      .line-err
        display: block
        font-size: 11px

    .single-form
      display: block
    .multi-form
      display: none
    .is-multi
      .single-form
        display: none
      .multi-form
        display: block

    .mix-with
      display: block
      margin-top: 16px
</style>
