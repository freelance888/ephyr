<script lang='js'>
  export let streamInfo;
  export let isError;
</script>

<template>
    <span
      class="info-icon"
      class:has-error={isError}
      uk-icon="icon: info; ratio: 0.7"
      uk-tooltip={streamInfo}
    />
</template>

<style lang='stylus'>
  .info-icon
    margin-left: 4px
</style>
