<script lang="ts">
  import { onDestroy } from 'svelte';
  import { mutation } from 'svelte-apollo';
  import { SetRestream } from '../../api/client.graphql';
  import { sanitizeLabel, showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';
  import { RestreamModel } from '../models/restream.model';
  import { writable } from 'svelte/store';
  import cloneDeep from 'lodash/cloneDeep';
  import isEqual from 'lodash/isEqual';
  import RestreamBackup from './RestreamBackup.svelte';

  const setRestreamMutation = mutation(SetRestream);

  export let visible = false;
  export let public_host = 'localhost';

  export let restream: RestreamModel = new RestreamModel();
  let previous: RestreamModel = cloneDeep(restream);

  let restreamStore = writable(restream);

  let submitable = false;
  onDestroy(
    restreamStore.subscribe((current) => {
      submitable = current.key !== '';
      let changed = !current.id;

      if (!!current.id) {
        changed ||=
          current.key !== previous.key ||
          current.label !== previous.label ||
          current.isPull !== previous.isPull;
      }

      if (current.isPull) {
        submitable &&= current.pullUrl !== '';
        if (!!current.id) {
          changed ||= current.pullUrl !== previous.pullUrl;
        }
      }

      if (current.backups.length !== previous.backups.length) {
        changed ||= true;
      } else {
        current.backups.forEach((x, i) => {
          changed ||= !isEqual(x, previous.backups[i]);
        });
      }

      if (!!current.id) {
        changed ||= current.withHls !== previous.withHls;
      }
      submitable &&= changed;
    })
  );

  async function submit(): Promise<void> {
    if (!submitable) return;

    let variables: unknown = {
      key: restream.key,
      with_hls: restream.withHls,
    };

    if (restream.label) {
      variables.label = restream.label;
    }

    if (restream.isPull) {
      variables.url = restream.pullUrl;
    }

    if (restream.backups.length) {
      variables.backup_inputs = restream.backups.map((x) => ({
        key: x.key,
        src: x.pullUrl,
      }));
    }

    if (restream.id) {
      variables.id = restream.id;
    }

    try {
      await setRestreamMutation({ variables });
      close();
    } catch (e) {
      showError(e.message);
    }
  }

  const close = () => {
    visible = false;
  };

  const removeBackup = (index: number) => {
    restreamStore.update((v) => {
      v.removeBackup(index);
      return v;
    });
  };

  const addBackup = () => {
    restreamStore.update((v) => {
      v.addBackup();
      return v;
    });
  };

  const onChangeLabel = () => {
    restreamStore.update((v) => {
      v.label = sanitizeLabel(v.label);
      return v;
    });
  };

  const onChangeRestreamKey = () => {
    restreamStore.update((v) => {
      v.key = sanitizeLabel(v.key);
      return v;
    });
  };

  const onChangeBackup = () => {
    restreamStore.update((v) => {
      return v;
    });
  };
</script>

<template>
  <div
    class="uk-modal uk-open"
    use:saveOrCloseByKeys={{ save: submit, close: close }}
  >
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">
        {#if $restreamStore.id}Edit{:else}Add new{/if} input source for re-streaming
      </h2>
      <button
        class="uk-modal-close-outside"
        uk-close
        type="button"
        on:click={close}
      />

      <fieldset>
        <div class="restream">
          <input
            class="uk-input uk-form-small"
            type="text"
            data-testid="add-input-modal:label-input"
            bind:value={$restreamStore.label}
            on:change={onChangeLabel}
            placeholder="optional label"
          />
          <label
            >rtmp://{public_host}/<input
              class="uk-input"
              type="text"
              data-testid="add-input-modal:stream-key-input"
              placeholder="<stream-key>"
              bind:value={$restreamStore.key}
              on:change={onChangeRestreamKey}
            />/origin</label
          >
          <div class="uk-alert">
            {#if $restreamStore.isPull}
              Server will pull RTMP stream from the address below.
              <br />
              Supported protocols:
              <code>rtmp://</code>,
              <code>http://.m3u8</code> (HLS)
            {:else}
              Server will await RTMP stream to be pushed onto the address above.
            {/if}
          </div>
        </div>
        <div class="pull">
          <label
            ><input
              class="uk-checkbox"
              type="checkbox"
              bind:checked={$restreamStore.isPull}
            /> or pull from</label
          >
          {#if $restreamStore.isPull}
            <input
              class="uk-input"
              type="text"
              bind:value={$restreamStore.pullUrl}
              placeholder="rtmp://..."
            />
          {/if}
        </div>
        <div class="hls">
          <label
            ><input
              class="uk-checkbox"
              type="checkbox"
              bind:checked={$restreamStore.withHls}
            /> with HLS endpoint</label
          >
        </div>

        <div class="uk-section uk-section-xsmall backups-section">
          <button
            class="uk-button uk-button-primary uk-button-small"
            on:click={() => addBackup()}
            >Add backup
          </button>
          <ul class="uk-list uk-margin-left">
            {#each $restreamStore.backups as backup, index}
              <RestreamBackup
                {backup}
                removeFn={() => removeBackup(index)}
                onChangeFn={() => onChangeBackup()}
              />
            {/each}
          </ul>
        </div>
      </fieldset>

      <button
        class="uk-button uk-button-primary"
        data-testid="add-input-modal:confirm"
        disabled={!submitable}
        on:click={submit}
      >
        {#if $restreamStore.id}Edit{:else}Add{/if}
      </button>
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

  .uk-alert
    font-size: 14px
    margin: 10px 0

  .restream
    .uk-form-small
      display: block
      width: auto
      margin-bottom: 15px

    label
      display: block

      input:not(.uk-form-small)
        display: inline
        width: auto
        margin-top: -5px

  .pull
    .uk-input
      margin-bottom: 10px

  .backups-section
    padding-top: 10px;
    padding-bottom: 0;

</style>
