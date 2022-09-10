<script lang='ts'>
  import { onDestroy, onMount } from 'svelte';
  import { mutation } from 'svelte-apollo';
  import { SetRestream } from '../../api/client.graphql';
  import { areObjectsEqual, showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';
  import { RestreamModel } from '../models/restream.model';
  import { writable } from 'svelte/store';
  import { get } from 'svelte/store';
  import cloneDeep from 'lodash/cloneDeep';

  const setRestreamMutation = mutation(SetRestream);

  export let visible = false;
  export let public_host = 'localhost';

  export let model: RestreamModel = new RestreamModel();
  let prevModel: RestreamModel = cloneDeep(model);

  let modelStore = writable(model);

  let submitable = false;
  onDestroy(
    modelStore.subscribe((v) => {
      submitable = !areObjectsEqual(get(modelStore), prevModel);
    })
  );

  // onDestroy(
  //   value.subscribe((v) => {
  //     submitable = v.key !== '';
  //     let changed = !v.edit_id;
  //     if (!!v.edit_id) {
  //       changed |=
  //         v.key !== v.prev_key ||
  //         v.label !== v.prev_label ||
  //         v.is_pull !== v.prev_is_pull ||
  //         v.with_backup !== v.prev_with_backup;
  //     }
  //     if (v.is_pull) {
  //       submitable &= v.pull_url !== '';
  //       if (!!v.edit_id) {
  //         changed |= v.pull_url !== v.prev_pull_url;
  //       }
  //     }
  //     if (v.with_backup) {
  //       if (!!v.edit_id) {
  //         changed |= v.backup_is_pull !== v.prev_backup_is_pull;
  //       }
  //       if (v.backup_is_pull) {
  //         submitable &= v.backup_pull_url !== '';
  //         if (!!v.edit_id) {
  //           changed |= v.backup_pull_url !== v.prev_backup_pull_url;
  //         }
  //       }
  //     }
  //     if (!!v.edit_id) {
  //       changed |= v.with_hls !== v.prev_with_hls;
  //     }
  //     submitable &= changed;
  //   })
  // );

  async function submit() {
    if (!submitable) return;

    let variables = {
      id: null as string | null,
      key: model.key,
      with_hls: model.withHls,
      with_backup: false,
      label: '',
      url: '',
      backup_url: ''
    };

    if (model.label !== '') {
      variables.label = model.label;
    }

    if (model.isPull) {
      variables.url = model.pullUrl;
    }

    if (model.withBackup) {
      variables.with_backup = true;
      if (model.backupIsPull) {
        variables.backup_url = model.backupPullUrl;
      }
    }

    if (model.id) {
      variables.id = model.id;
    }

    try {
      await setRestreamMutation({ variables });
      close();
    } catch (e) {
      showError(e.message);
    }
  }

  function close() {
    visible = false;
  }
</script>

<template>
  <div>NEW MODAL</div>
  <div
    class='uk-modal uk-open'
    use:saveOrCloseByKeys={{ save: submit, close: close }}
  >
    <div class='uk-modal-dialog uk-modal-body'>
      <h2 class='uk-modal-title'>
        {#if $modelStore.id}Edit{:else}Add new{/if} input source for re-streaming
      </h2>
      <button
        class='uk-modal-close-outside'
        uk-close
        type='button'
        on:click={close}
      />

      <fieldset>
        <div class='restream'>
          <input
            class='uk-input uk-form-small'
            type='text'
            data-testid='add-input-modal:label-input'
            bind:value={$modelStore.label}
            on:change={() => $modelStore.sanitizeLabel()}
            placeholder='optional label'
          />
          <label
          >rtmp://{public_host}/<input
            class='uk-input'
            type='text'
            data-testid='add-input-modal:stream-key-input'
            placeholder='<stream-key>'
            bind:value={$modelStore.key}
          />/origin</label
          >
          <div class='uk-alert'>
            {#if $modelStore.isPull}
              Server will pull RTMP stream from the address below.
              <br />
              Supported protocols:
              <code>rtmp://</code>,
              <code>http://.m3u8</code> (HLS)
            {:else}
              Server will await RTMP stream to be pushed onto the address
              above.
            {/if}
          </div>
        </div>
        <div class='pull'>
          <label
          ><input
            class='uk-checkbox'
            type='checkbox'
            bind:checked={$modelStore.isPull}
          /> or pull from</label
          >
          {#if $modelStore.isPull}
            <input
              class='uk-input'
              type='text'
              bind:value={$modelStore.pullUrl}
              placeholder='rtmp://...'
            />
          {/if}
        </div>
        <div class='backup'>
          <label
          ><input
            class='uk-checkbox'
            type='checkbox'
            bind:checked={$modelStore.withBackup}
          /> with backup</label
          >
          {#if $modelStore.withBackup}
            <label
            ><input
              class='uk-checkbox'
              type='checkbox'
              bind:checked={$modelStore.backupIsPull}
            /> pulled from</label
            >
            {#if $modelStore.backupIsPull}
              <input
                class='uk-input'
                type='text'
                bind:value={$modelStore.backupPullUrl}
                placeholder='rtmp://...'
              />
            {/if}
          {/if}
        </div>
        <div class='hls'>
          <label
          ><input
            class='uk-checkbox'
            type='checkbox'
            bind:checked={$modelStore.withHls}
          /> with HLS endpoint</label
          >
        </div>
      </fieldset>

      <button
        class='uk-button uk-button-primary'
        data-testid='add-input-modal:confirm'
        disabled={!submitable}
        on:click={submit}
      >
        {#if $modelStore.id}Edit{:else}Add{/if}
      </button
      >
    </div>
  </div>
</template>

<style lang='stylus'>
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

  .backup
    label + label
      margin-left: 15px
</style>
