<script lang="js">
  export let count;
  export let active;
  export let status;
  export let disabled;
  export let title;
  export let handleClick = () => {};
  function _handleClick(e) {
    // TODO: What is happening here?
    if (disabled) {
      return;
    }

    handleClick(e);
  }
</script>

<template>
  <div class="status-filter" on:keydown={_handleClick} on:click={_handleClick}>
    <div
      title={title ? title : status}
      class="content"
      class:active
      class:disabled
      class:online={status === 'ONLINE'}
      class:offline={status === 'OFFLINE'}
      class:initializing={status === 'INITIALIZING'}
      class:unstable={status === 'UNSTABLE'}
    >
      <span class="circle" />
      {count}
    </div>
  </div>
</template>

<style lang="stylus">
  .status-filter
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
