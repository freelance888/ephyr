<script lang="js">
  export let count;
  export let active;
  export let status;
  export let disabled;
  export let title;
  export let handleClick = () => {};
</script>

<template>
  <div
    class="status-filter"
    on:click={(e) => {
      if (disabled) {
        return;
      }

      handleClick(e);
    }}
  >
    <div
      {title}
      class="content"
      class:active
      class:disabled
      class:online={status === 'ONLINE'}
      class:offline={status === 'OFFLINE'}
      class:initializing={status === 'INITIALIZING'}
    >
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
      &.initializing
        color: #faa05a
      &.online
        color: #32d296
      &.offline
        color: #f0506e;
      &.active
        background-color: #cecece
      &.disabled
        &:hover
          cursor: not-allowed
      &:hover
        background-color: #bdbdbd
        cursor: pointer
</style>
