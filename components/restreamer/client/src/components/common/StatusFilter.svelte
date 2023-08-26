<script lang="js">
  import Fa from 'svelte-fa';
  import { faInfoCircle } from '@fortawesome/free-solid-svg-icons/faInfoCircle';
  import { STREAM_ERROR, STREAM_WARNING } from '../../utils/constants';

  export let count;
  export let active;
  export let status;
  export let disabled;
  export let title;
  export let handleClick = () => {};

  $: iconClass = getClass(status);

  function getClass(s) {
    let cls = 'info-icon';

    if (s === STREAM_ERROR) {
      cls += ' streams-errors';
    } else if (s === STREAM_WARNING) {
      cls += ' streams-warnings'
    }

    return cls;
  }

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
        <Fa class={iconClass} icon={faInfoCircle}></Fa>
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

  :global(.streams-errors)
    color: var(--danger-color)
  :global(.streams-warnings)
    color: var(--warning-color)

</style>
