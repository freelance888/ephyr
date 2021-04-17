<script lang="js">
  let showDialog = false;
  let functionToCall = {
    func: null,
    args: null,
  };

  function callFunction() {
    showDialog = false;
    functionToCall['func'](...functionToCall['args']);
  }

  function confirm(func, ...args) {
    functionToCall = { func, args };
    showDialog = true;
  }
</script>

<template>
  <slot {confirm} />

  {#if showDialog}
    <div class="uk-modal uk-open">
      <div class="uk-modal-dialog uk-modal-body">
        <h2 class="uk-modal-title">
          <slot name="title">
            Are you sure you want to perform this action?
          </slot>
        </h2>
        <button
          class="uk-modal-close-outside"
          uk-close
          type="button"
          on:click={() => (showDialog = false)}
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
