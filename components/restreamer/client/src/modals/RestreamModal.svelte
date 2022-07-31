<script lang="js">
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { mutation } from 'svelte-apollo';
  import { SetRestream } from '../../api/client.graphql';
  import { showError } from '../utils/util';
  import { restreamModal as value } from '../stores';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const setRestreamMutation = mutation(SetRestream);

  export let public_host = 'localhost';

  let submitable = false;
  onDestroy(
    value.subscribe((v) => {
      submitable = v.key !== '';
      let changed = !v.edit_id;
      if (!!v.edit_id) {
        changed |=
          v.key !== v.prev_key ||
          v.label !== v.prev_label ||
          v.is_pull !== v.prev_is_pull ||
          v.with_backup !== v.prev_with_backup;
      }
      if (v.is_pull) {
        submitable &= v.pull_url !== '';
        if (!!v.edit_id) {
          changed |= v.pull_url !== v.prev_pull_url;
        }
      }
      if (v.with_backup) {
        if (!!v.edit_id) {
          changed |= v.backup_is_pull !== v.prev_backup_is_pull;
        }
        if (v.backup_is_pull) {
          submitable &= v.backup_pull_url !== '';
          if (!!v.edit_id) {
            changed |= v.backup_pull_url !== v.prev_backup_pull_url;
          }
        }
      }
      if (!!v.edit_id) {
        changed |= v.with_hls !== v.prev_with_hls;
      }
      submitable &= changed;
    })
  );

  async function submit() {
    if (!submitable) return;
    const v = get(value);

    let variables = { key: v.key, with_hls: v.with_hls, with_backup: false };
    if (v.label !== '') {
      variables.label = v.label;
    }
    if (v.is_pull) {
      variables.url = v.pull_url;
    }
    if (v.with_backup) {
      variables.with_backup = true;
      if (v.backup_is_pull) {
        variables.backup_url = v.backup_pull_url;
      }
    }
    if (v.edit_id) {
      variables.id = v.edit_id;
    }

    try {
      await setRestreamMutation({ variables });
      close();
    } catch (e) {
      showError(e.message);
    }
  }

  function close() {
    value.close();
  }
</script>

<template>
  {#if $value.visible}
    <div
      class="uk-modal uk-open"
      use:saveOrCloseByKeys={{ save: submit, close: close }}
    >
      <div class="uk-modal-dialog uk-modal-body">
        <h2 class="uk-modal-title">
          {#if $value.edit_id}Edit{:else}Add new{/if} input source for re-streaming
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
              bind:value={$value.label}
              on:change={() => value.sanitizeLabel()}
              placeholder="optional label"
            />
            <label
              >rtmp://{public_host}/<input
                class="uk-input"
                type="text"
                data-testid="add-input-modal:stream-key-input"
                placeholder="<stream-key>"
                bind:value={$value.key}
              />/origin</label
            >
            <div class="uk-alert">
              {#if $value.is_pull}
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
          <div class="pull">
            <label
              ><input
                class="uk-checkbox"
                type="checkbox"
                bind:checked={$value.is_pull}
              /> or pull from</label
            >
            {#if $value.is_pull}
              <input
                class="uk-input"
                type="text"
                bind:value={$value.pull_url}
                placeholder="rtmp://..."
              />
            {/if}
          </div>
          <div class="backup">
            <label
              ><input
                class="uk-checkbox"
                type="checkbox"
                bind:checked={$value.with_backup}
              /> with backup</label
            >
            {#if $value.with_backup}
              <label
                ><input
                  class="uk-checkbox"
                  type="checkbox"
                  bind:checked={$value.backup_is_pull}
                /> pulled from</label
              >
              {#if $value.backup_is_pull}
                <input
                  class="uk-input"
                  type="text"
                  bind:value={$value.backup_pull_url}
                  placeholder="rtmp://..."
                />
              {/if}
            {/if}
          </div>
          <div class="hls">
            <label
              ><input
                class="uk-checkbox"
                type="checkbox"
                bind:checked={$value.with_hls}
              /> with HLS endpoint</label
            >
          </div>
        </fieldset>

        <button
          class="uk-button uk-button-primary"
          data-testid="add-input-modal:confirm"
          disabled={!submitable}
          on:click={submit}
          >{#if $value.edit_id}Edit{:else}Add{/if}</button
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
