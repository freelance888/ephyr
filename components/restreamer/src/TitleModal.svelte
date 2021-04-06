<script lang="js">
  import { mutation } from 'svelte-apollo';

  import { SetTitle } from './api/graphql/client.graphql';

  import { showError } from './util';

  const setTitleMutation = mutation(SetTitle);

  export let visible = false;
  export let title;

  function close() {
    visible = false;
  }

  async function submit_change() {
    try {
      await setTitleMutation({ variables: { title }});
      close();
      //window.location.reload();
    } catch (e) {
      showError(e.message);
    }
  }
</script>

<template>
  <div class="uk-modal" class:uk-open={visible}>
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">Change title</h2>
      <button
        class="uk-modal-close-outside"
        uk-close
        type="button"
        on:click={close}
      />
      <fieldset class="single-form">
        <input class="uk-input" bind:value={title} placeholder="Title" />
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

    fieldset
      border: none

</style>
