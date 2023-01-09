<script lang="js">
  import { STREAM_ERROR, STREAM_WARNING } from '../../utils/constants';

  export let count;
  export let active;
  export let status;
  export let disabled;
  export let title;
  export let handleClick = () => {};
</script>

<template>
  <!-- svelte-ignore a11y-click-events-have-key-events -->
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
      title={title ? title : status}
      class="content"
      class:active
      class:disabled
      class:online={status === 'ONLINE'}
      class:offline={status === 'OFFLINE'}
      class:initializing={status === 'INITIALIZING'}
      class:unstable={status === 'UNSTABLE'}
    >
      {#if [STREAM_ERROR, STREAM_WARNING].includes(status)}
        <i
          class:streams-errors={status === STREAM_ERROR}
          class:streams-warnings={status === STREAM_WARNING}
          class="fa fa-info-circle info-icon"
        />
      {:else}
        <span class="circle" />
      {/if}
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

  .streams-errors
    color: var(--danger-color)
  .streams-warnings
    color: var(--warning-color)
</style>
