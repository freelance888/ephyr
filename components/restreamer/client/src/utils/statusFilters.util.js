export const toggleFilterStatus = (filters, filter) => {
  const filterIndex = filters.indexOf(filter);

  return filterIndex === -1
    ? [...filters, filter]
    : filters.filter((item) => item !== filter);
};
