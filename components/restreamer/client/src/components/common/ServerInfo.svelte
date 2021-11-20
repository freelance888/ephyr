<script lang="js">
  export let serverInfo;

  const formatMem = (value) => {
    return value.toLocaleString('en-US', {maximumFractionDigits:0});
  }

  const formatNet = (value) => {
    return value.toLocaleString('en-US', {maximumFractionDigits:2});
  }
</script>

<template>
  {#if serverInfo}

      <div class="statistics-value">CPU - {serverInfo.cpuUsage.toFixed()}%</div>
      <div class="statistics-value">MEM - {formatMem(serverInfo.ramTotal)}Mb / {formatMem(serverInfo.ramFree)}Mb</div>
      <div class="statistics-value">NET - <i class="fas fa-arrow-up"/>{formatNet(serverInfo.txDelta)} Mbit/s, <i class="fas fa-arrow-down"/> {formatNet(serverInfo.rxDelta)} Mbit/s</div>

  {/if}
</template>

<style lang="stylus">
  .statistics-value
    margin-left: 10px;
    margin-right: 10px;
    //&:not(:last-child):after {
    //  content1: "|"
    //  display: inline-block;
    //}
</style>
