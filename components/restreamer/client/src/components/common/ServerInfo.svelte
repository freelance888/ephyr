<script lang="js">
  import Fa from 'svelte-fa';
  import { isNumber } from '../../utils/util';
  import { faInfoCircle } from '@fortawesome/free-solid-svg-icons/faInfoCircle';

  export let serverInfo;
  export let rowMode = false;

  const formatMem = (value) => {
    return isNumber(value)
      ? value.toLocaleString('en-US', { maximumFractionDigits: 0 })
      : '';
  };

  const formatNet = (value) => {
    return isNumber(value) ? value.toFixed(1) : '';
  };

  const formatInteger = (value) => {
    return isNumber(value) ? value.toFixed() : '';
  };

  const formatErrorMsg = (value) => {
    return value ? value.substring(0, 100) : '';
  };

  const formatCoresText = (value) => {
    return value === 1 ? 'core' : 'cores';
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
          >{formatInteger(serverInfo.cpuUsage)}% ({formatInteger(
            serverInfo.cpuCores
          )}
          {formatCoresText(serverInfo.cpuCores)})</span
        >
      </div>
      <div class="server-info-row">
        <span class="title">MEM</span> -
        <span class="value uk-text-muted" title="Total memory / Free memory"
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
        {#if serverInfo.errorMsg}
          <span
            class="error-icon value uk-text-danger"
            title={formatErrorMsg(serverInfo.errorMsg)}
          >
            <Fa faInfoCircle icon={faInfoCircle} />
          </span>
        {/if}
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
  .error-icon
    margin-left: 4px;
</style>
