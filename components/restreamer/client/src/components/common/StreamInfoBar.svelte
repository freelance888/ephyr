<script lang="js">
import { streamInfoList } from '../../utils/constants';
import StatusFilter from './StatusFilter.svelte';
</script>

<template>
      <span class="toolbar-label">
        STREAMS:
        {#each streamInfoList as status (status)}
          <StatusFilter
            {status}
            count={1}
          />
        {/each}
      </span>
</template>
