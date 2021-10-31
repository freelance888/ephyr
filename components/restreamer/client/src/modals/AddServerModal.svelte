<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { AddClient } from '../../api/dashboard.graphql';
  import { showError } from '../utils/util';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const addClientMutation = mutation(AddClient);

  export let visible = false;
  let host = '';

  function close() {
    visible = false;
  }

  async function submit_change() {
    try {
      await addClientMutation({ variables: { host } });
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
      <h2 class="uk-modal-title">Add server</h2>
      <button
        class="uk-modal-close-outside"
        uk-close
        type="button"
        on:click={close}
      />
      <fieldset class="settings-form">
        <input class="uk-input" bind:value={host} placeholder="Server name" />
        <div class="uk-alert">
          IP address or host name of the server for getting statistics info.
        </div>
      </fieldset>

      <button class="uk-button uk-button-primary" on:click={submit_change}
        >Add</button
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
