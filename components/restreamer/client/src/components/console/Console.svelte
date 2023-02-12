<script lang="ts">
 type ConsoleItem = {
  type: 'error' | 'warning',
  message: string
 };

 let items: ConsoleItem[] = [
  {
   type: 'error',
   message: 'Can\'t create vertical or horizontal splits you only need the Split component.'
  },
  {
   type: 'warning',
   message: 'The default creates a left(50%) | right(50%) split, no minimum pane sizes, and renders the default splitter.'
  },
 ]
</script>

<template>
<ul>
 <li>
  {#each items as item}
   { item.message }
  {/each}
 </li>
</ul>
</template>

<style lang="stylus">

</style>
