<script lang="js">
  export let disabled;
  export let active;
  export let handleClick = () => {};
</script>

<template>
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <div
    class="toggle-container"
    on:click={(e) => {
      if (disabled) {
        return;
      }
      handleClick(e);
    }}
  >
    <div class:active class:disabled class="content">
      <slot />
    </div>
  </div>
</template>

<style lang="stylus">
  .toggle-container
    min-width: 32px
    display: inline-flex

    .content
      width: 100%
      text-align: center
      margin-right: 2px
      background-color: inherit
      padding: 1px 4px
      border-radius: 2px
      outline: none
      &.active
        background-color: #cecece
      &.disabled
        &:hover
          cursor: not-allowed
      &:hover
        background-color: #bdbdbd
        cursor: pointer

</style>
