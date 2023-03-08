<script lang="js">
  import { onDestroy } from 'svelte';
  import { mutation, subscribe } from 'svelte-apollo';
  import { SetRestream, Info } from '../../api/client.graphql';
  import { sanitizeLabel, showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';
  import { RestreamModel } from '../models/restream.model';
  import { writable } from 'svelte/store';
  import cloneDeep from 'lodash/cloneDeep';
  import isEqual from 'lodash/isEqual';
  import RestreamBackup from './RestreamBackup.svelte';

  const info = subscribe(Info, { errorPolicy: 'all' });
  const setRestreamMutation = mutation(SetRestream);

  export let visible = false;
  export let public_host = 'localhost';
  export let restream = new RestreamModel();

  let previous = cloneDeep(restream);
  let restreamStore = writable(restream);
  $: fileIdToolTip = !hasApiKey
    ? 'Please specify Google Api Key in `Settings` before setting File ID'
    : null;

  $: hasApiKey = $info.data?.info?.googleApiKey;

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

      if (current.maxFilesInPlaylist ?? '' !== previous.maxFilesInPlaylist) {
        changed ||= true;
      }

      if (current.fileId !== previous.fileId) {
        changed ||= true;
      }

      if (!!current.id) {
        changed ||= current.withHls !== previous.withHls;
      }
      submitable &&= changed;
    })
  );

  async function submit() {
    if (!submitable) return;

    let variables = {
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

    if (restream.fileId) {
      variables.file_id = restream.fileId;
    }

    if (restream.maxFilesInPlaylist) {
      variables.max_files_in_playlist = restream.maxFilesInPlaylist;
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

  const removeBackup = (index) => {
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
            />/primary</label
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
            data-testid="add-output-modal:add-backup"
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

        <div class="uk-section uk-section-xsmall">
          <div
            class="uk-position-relative"
            class:question-pointer={!hasApiKey}
            uk-tooltip={fileIdToolTip}
          >
            <label class="uk-flex uk-flex-between backup-item">
              <span class="label-file-id" class:disabled={!hasApiKey}
                >File backup</span
              >
              <input
                class="uk-input file-id"
                type="text"
                bind:value={$restreamStore.fileId}
                disabled={!hasApiKey}
                placeholder="Google File ID"
              />
            </label>
            <button
              type="button"
              class="clear-file-id uk-position-absolute"
              uk-close
              on:click={() => ($restreamStore.fileId = '')}
            />
          </div>
          <div
            class="uk-alert uk-position-relative"
            class:question-pointer={!hasApiKey}
            uk-tooltip={fileIdToolTip}
          >
            Max number of files in a playlist.
            <input
              class="uk-input uk-width-1-4 files-limit uk-position-absolute"
              type="number"
              min="2"
              step="1"
              bind:value={$restreamStore.maxFilesInPlaylist}
              placeholder="Files limit"
              disabled={!hasApiKey}
            />
          </div>
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

  .uk-section>:last-child
    margin-top: 10px;

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

  .files-limit
    top: 50%;
    transform: translateY(-50%);
    right 10px;

  .clear-file-id
    top: 50%
    transform: translateY(-50%)
    right: 8px

  .label-file-id
    margin-left: auto
    align-self: center
    font-size: 0.875em

  .file-id
    width: 59%

  .backup-item
    column-gap: 20px
  
  .disabled
    color: #999

  .question-pointer:hover
    .uk-input
      background: #e5e5e5
      cursor: help

</style>
