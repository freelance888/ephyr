<script lang="js">
  import Fa from 'svelte-fa';
  import { faInfoCircle } from '@fortawesome/free-solid-svg-icons/faInfoCircle';
  import { STREAM_ERROR, STREAM_WARNING } from '../../utils/constants';
  import ToggleButton from './ToggleButton.svelte';

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
  <ToggleButton {handleClick} {disabled} {active}>
    <div
      title={title ? title : status}
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
  </ToggleButton>
</template>

<style lang="stylus">
  :global(.streams-errors)
    color: var(--danger-color)
  :global(.streams-warnings)
    color: var(--warning-color)

</style>
