<script lang="js">

  export let streamsErrorsTooltip;
  export let streamsDiffTooltip;

</script>

<template>
  {#key streamsErrorsTooltip || streamsDiffTooltip}
    <span>
      <i
        class="fa fa-info-circle info-icon"
        class:has-error={!!streamsErrorsTooltip}
        class:has-warning={!!streamsDiffTooltip}
        class:hidden={!streamsErrorsTooltip && !streamsDiffTooltip}
        uk-tooltip={streamsErrorsTooltip || streamsDiffTooltip}
      />
    </span>
  {/key}
</template>

<style lang="stylus">
  .info-icon
    font-size: 16px
</style>
