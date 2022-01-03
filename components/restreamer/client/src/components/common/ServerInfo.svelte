<script lang="js">
  export let serverInfo;
  export let rowMode = false;

  const formatMem = (value) => {
    return value
      ? value.toLocaleString('en-US', { maximumFractionDigits: 0 })
      : '';
  };

  const formatNet = (value) => {
    return value ? value.toFixed(1) : '';
  };

  const formatCpuUsage = (value) => {
    return value ? value.toFixed() : '';
  };
</script>

<template>
  {#if serverInfo}
    <div
      class="server-info uk-flex-middle uk-text-small"
      class:uk-flex-inline={rowMode}
    >
      <div class="server-info-row">
        <span class="title">CPU</span> -
        <span class="value uk-text-muted" title="CPU usage"
          >{formatCpuUsage(serverInfo.cpuUsage)}%</span
        >
      </div>
      <div class="server-info-row">
        <span class="title">MEM</span> -
        <span class="value uk-text-muted" title="Total memory / Used memory"
          >{formatMem(serverInfo.ramTotal)} Mb / {formatMem(serverInfo.ramFree)}
          Mb</span
        >
      </div>
      <div class="server-info-row">
        <span class="title">NET</span> -
        <span
          class="value uk-text-muted"
          title="Network: send⬆️, receive⬇️ speed (megabytes/second)"
          >⬆️ {formatNet(serverInfo.txDelta)} Mb/s, ⬇ {formatNet(
            serverInfo.rxDelta
          )} Mb/s</span
        >
      </div>
    </div>
  {/if}
</template>

<style lang="stylus">
  .server-info
    height: 38px;
  .server-info-row
    margin-left: 10px
    margin-right: 10px
  .value
    cursor: pointer
</style>
