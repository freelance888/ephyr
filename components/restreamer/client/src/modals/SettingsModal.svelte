<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { SetSettings } from '../../api/client.graphql';
  import { showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const setSettingsMutation = mutation(SetSettings);

  export let visible = false;
  export let info;

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
</script>

<template>
  <div
    class="uk-modal uk-open"
    use:saveOrCloseByKeys={{ save: submit_change, close: close }}
  >
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">Change settings</h2>
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
        <label
          ><input
            class="uk-checkbox"
            bind:checked={info.deleteConfirmation}
            type="checkbox"
          /> Confirm deletion</label
        >
        <div class="uk-alert">
          Ask for confirmation when we delete input(s) or output(s)
        </div>
        <label
          ><input
            class="uk-checkbox"
            bind:checked={info.enableConfirmation}
            type="checkbox"
          /> Confirm enabling/disabling</label
        >
        <div class="uk-alert">
          Ask for confirmation when we enable/disable input(s) or output(s)
        </div>
        <input
          min="0"
          step="1"
          max="120"
          class="uk-input uk-width-small"
          bind:value={info.outputStarDelay}
          type="number"
        />
        <div class="uk-alert">
          <span>Delay before starting next output in</span> <span class='uk-text-bold'>seconds</span><span>. Min = 0. Max = 120.</span>
        </div>
      </fieldset>

      <button class="uk-button uk-button-primary" on:click={submit_change}
        >Change</button
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

      & >.uk-alert
        margin-top: 5px !important;
</style>
