<script lang='ts'>
  import { onDestroy } from 'svelte';
  import { mutation } from 'svelte-apollo';
  import { SetRestream } from '../../api/client.graphql';
  import { showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';
  import { RestreamModel } from '../models/restream.model';
  import { writable } from 'svelte/store';
    import cloneDeep from 'lodash/cloneDeep';

  const setRestreamMutation = mutation(SetRestream);

  export let visible = false;
  export let public_host = 'localhost';

  export let model: RestreamModel = new RestreamModel();
  let previous: RestreamModel = cloneDeep(model);

  let modelStore = writable(model);

  let submitable = false;
  onDestroy(
    modelStore.subscribe((current) => {

      submitable = current.key !== '';
      let changed = !current.id;

      if (!!current.id) {
        changed |=
          current.key !== previous.key ||
          current.label !== previous.label ||
          current.isPull !== previous.isPull ||
          current.withBackup !== previous.withBackup;
      }

      if (current.isPull) {
        submitable &= current.pullUrl !== '';
        if (!!current.id) {
          changed |= current.pullUrl !== previous.pullUrl;
        }
      }

      if (current.withBackup) {
        if (!!current.id) {
          changed |= current.backupIsPull !== previous.backupIsPull;
        }
        if (current.backupIsPull) {
          submitable &= current.backupPullUrl !== '';
          if (!!current.id) {
            changed |= current.backupPullUrl !== previous.backupPullUrl;
          }
        }
      }

      if (!!current.id) {
        changed |= current.withHls !== previous.withHls;
      }
      submitable &= changed;
    })
  );

   async function submit() {
    if (!submitable) return;

    let variables: unknown = {
      key: model.key,
      with_hls: model.withHls,
      with_backup: false,
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
        <div class='hls'>
          <label
          ><input
            class='uk-checkbox'
            type='checkbox'
            bind:checked={$modelStore.withHls}
          /> with HLS endpoint</label
          >
        </div>

        <div class="uk-section uk-section-xsmall">
          <button class="uk-button uk-button-primary uk-button-small">Add backup</button>
          <ul class="uk-list uk-list-divider uk-margin-left">
            <li>List item 1</li>
            <li>List item 2</li>
            <li>List item 3</li>
          </ul>
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
