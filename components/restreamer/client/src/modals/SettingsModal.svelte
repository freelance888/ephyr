<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { SetSettings } from '../../api/client.graphql';
  import { showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const setSettingsMutation = mutation(SetSettings);

  export let visible = false;
  export let info;

  $: fileIdToolTip = !info.googleApiKey
    ? 'Please specify Google Api Key in `Settings` before setting File ID'
    : null;

  function close() {
    visible = false;
  }

  async function submit_change() {
    try {
      await setSettingsMutation({ variables: info });
      close();
    } catch (e) {
      showError(e.message);
    }
  }

  function onEmptyGooleApiKey() {
    if (!info.googleApiKey) info.maxFilesInPlaylist = null;
  }
</script>

<template>
  <div
    class="uk-modal uk-open"
    use:saveOrCloseByKeys={{ save: submit_change, close: close }}
  >
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">Settings</h2>
      <button
        class="uk-modal-close-outside"
        uk-close
        type="button"
        on:click={close}
      />
      <fieldset class="settings-form">
        <input class="uk-input" bind:value={info.title} placeholder="Title" />
        <div class="uk-alert">
          Title for the server. This title is visible in current tab of the
          browser
        </div>
        <label class="uk-display-block"
          ><input
            class="uk-checkbox"
            bind:checked={info.deleteConfirmation}
            type="checkbox"
          /> Confirm deletion</label
        >
        <label class="uk-display-block"
          ><input
            class="uk-checkbox"
            bind:checked={info.enableConfirmation}
            type="checkbox"
          /> Confirm enabling/disabling</label
        >
        <input
          class="uk-input google-api-key"
          type="password"
          bind:value={info.googleApiKey}
          on:input={onEmptyGooleApiKey}
          placeholder="Google API key"
        />
        <div class="uk-alert">
          Google API key for downloading video and audio files. It is necessary
          for file inputs.
        </div>
        <input
          class="uk-input uk-width-1-4"
          class:question-pointer={!info.googleApiKey}
          type="number"
          min="2"
          step="1"
          bind:value={info.maxFilesInPlaylist}
          placeholder="Files limit"
          disabled={!info.googleApiKey}
          uk-tooltip={fileIdToolTip}
        />
        <div class="uk-alert">Max amount of files in a playlist.</div>
      </fieldset>

      <button class="uk-button uk-button-primary" on:click={submit_change}
        >Confirm</button
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

    .settings-form
      border: none

    .google-api-key
      margin-top: 5px;

    .question-pointer:hover
      cursor: help
      background: #e5e5e5

</style>
