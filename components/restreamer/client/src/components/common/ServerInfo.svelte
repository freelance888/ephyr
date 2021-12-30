<script lang="ts">
  export let serverInfo;

  const formatMem = (value) => {
    return value ? value.toFixed() : '';
  }

  const formatNet = (value) => {
    return value ? value.toFixed() : '';
  }

  const formatCpuUsage = (value) => {
    return value ? value.toFixed() : '';
  }
</script>

<template>
  {#if serverInfo}

      <div class="statistics-value">CPU - {formatCpuUsage(serverInfo.cpuUsage)}%</div>
      <div class="statistics-value">MEM - {formatMem(serverInfo.ramTotal)}Mb / {formatMem(serverInfo.ramFree)}Mb</div>
      <div class="statistics-value">NET - <i class="fas fa-arrow-up"/> {formatNet(serverInfo.txDelta)} Mbit/s, <i class="fas fa-arrow-down"/> {formatNet(serverInfo.rxDelta)} Mbit/s</div>

  {/if}
</template>

<style lang="stylus">
  .statistics-value
    margin-left: 10px;
    margin-right: 10px;
    //&:not(:last-child):after {x
    //  content1: "|"
    //  display: inline-block;
    //}
</style>
