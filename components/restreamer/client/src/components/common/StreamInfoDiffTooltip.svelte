<script lang="js">
  import Fa from 'svelte-fa';
  import { faInfoCircle } from '@fortawesome/free-solid-svg-icons';

  export let streamsErrorsTooltip;
  export let streamsDiffTooltip;
</script>

<template>
  {#key streamsErrorsTooltip || streamsDiffTooltip}
    <span
      class="info-icon"
      class:has-error={!!streamsErrorsTooltip}
      class:has-warning={!!streamsDiffTooltip}
      class:hidden={!streamsErrorsTooltip && !streamsDiffTooltip}
      uk-tooltip={streamsErrorsTooltip || streamsDiffTooltip}
    >
      <Fa icon={faInfoCircle} />
    </span>
  {/key}
</template>

<style lang="stylus">
  .info-icon
    font-size: 16px
</style>
