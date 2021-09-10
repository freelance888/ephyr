<script lang="js">
  import { saveOrCloseByKeys } from '../../utils/directives.util';

  let showDialog = false;
  let functionToCall = {
    func: null,
    args: null,
  };

  function callFunction() {
    close();
    functionToCall['func'](...functionToCall['args']);
  }

  function confirm(func, ...args) {
    functionToCall = { func, args };
    showDialog = true;
  }

  function close() {
    showDialog = false;
  }
</script>

<template>
  <slot {confirm} />

  {#if showDialog}
    <div
      use:saveOrCloseByKeys={{ save: callFunction, close: close }}
      class="uk-modal uk-open"
    >
      <div class="uk-modal-dialog uk-modal-body uk-text-left">
        <h2 class="uk-modal-title">
          <slot name="title">
            Are you sure you want to perform this action?
          </slot>
        </h2>
        <button
          class="uk-modal-close-outside"
          uk-close
          type="button"
          on:click={close}
        />

        <p><slot name="description">This action can't be undone!</slot></p>

        <p class="uk-text-right">
          <button
            class="uk-button uk-button-default uk-modal-close"
            on:click={() => (showDialog = false)}>Cancel</button
          ><button class="uk-button uk-button-primary" on:click={callFunction}
            ><slot name="confirm">Confirm</slot></button
          >
        </p>
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

  .uk-modal-dialog
    > p
      font-size: 16px

  .uk-text-right
    button + button
      margin-left: 15px
</style>
