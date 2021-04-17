<script lang="js">
  import { mutation } from 'svelte-apollo';

  import { SetPassword } from './api/graphql/client.graphql';

  import { showError } from './util';

  const setPasswordMutation = mutation(SetPassword);

  export let visible = false;
  export let current_hash;
  let new_password = '';
  let rep_password = '';
  let old_password = '';

  $: change_submitable =
    new_password !== '' &&
    new_password === rep_password &&
    ((old_password !== '' && !!current_hash) || !current_hash);
  $: remove_submitable = !!current_hash && old_password !== '';

  function close() {
    visible = false;
    new_password = '';
    rep_password = '';
    old_password = '';
  }

  async function submit_change() {
    if (!change_submitable) return;

    let vars = { new: new_password };
    if (!!current_hash) {
      vars.old = old_password;
    }

    try {
      await setPasswordMutation({ variables: vars });
      close();
      window.location.reload();
    } catch (e) {
      showError(e.message);
    }
  }

  async function submit_remove() {
    if (!remove_submitable) return;
    try {
      await setPasswordMutation({ variables: { old: old_password } });
      close();
    } catch (e) {
      showError(e.message);
    }
  }
</script>

<template>
  <div class="uk-modal" class:uk-open={visible}>
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">
        {#if !current_hash}
          Protect page with password
        {:else}
          Change password
        {/if}
      </h2>
      <button
        class="uk-modal-close-outside"
        uk-close
        type="button"
        on:click={close}
      />

      <fieldset class="single-form">
        <input
          class="uk-input"
          type="password"
          bind:value={new_password}
          placeholder="{!current_hash ? 'P' : 'New p'}assword..."
        />
        <input
          class="uk-input"
          type="password"
          bind:value={rep_password}
          placeholder="Repeat the password..."
        />
        {#if !!current_hash}
          <input
            class="uk-input old"
            type="password"
            bind:value={old_password}
            placeholder="Current password for confirmation..."
          />
        {/if}
        {#if !current_hash}
          <div class="uk-alert">
            After setting a password the page will be protected with Basic HTTP
            Authentication.
          </div>
        {/if}
      </fieldset>

      <button
        class="uk-button uk-button-primary"
        disabled={!change_submitable}
        on:click={submit_change}>{!current_hash ? 'Add' : 'Change'}</button
      >
      {#if !!current_hash}
        <button
          class="uk-button uk-button-primary"
          disabled={!remove_submitable}
          on:click={submit_remove}>Remove</button
        >
      {/if}
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

      input
        & + input
          margin-top: 5px
        &.old
          margin-top: 20px

      .uk-alert
        font-size: 14px
        margin-bottom: 0

    .uk-button + .uk-button
      margin-right: 30px
</style>
