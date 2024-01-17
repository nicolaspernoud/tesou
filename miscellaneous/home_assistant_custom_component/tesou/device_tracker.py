"""Tesou device tracker."""

from homeassistant.components.device_tracker import SourceType
from homeassistant.components.device_tracker.config_entry import TrackerEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .api import TesouApi, User
from .const import DOMAIN


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up a tesou device tracker."""
    api: TesouApi = hass.data[DOMAIN][entry.entry_id]
    users: list[User] = await api.get_users()
    async_add_entities(
        (TesouDeviceTracker(user, entry, api) for user in users), update_before_add=True
    )


class TesouDeviceTracker(TrackerEntity):
    """Tesou device tracker."""

    _attr_has_entity_name = True

    def __init__(self, user: User, entry, api: TesouApi, data=None) -> None:
        """Set up Tesou entity."""
        self.user: User = user
        self.name = f"{user.name}Tracker"
        self._attr_unique_id = user.name.lower()
        self._entry = entry
        self.api = api
        self._data = data
        self.latlong: tuple[float, float] = (0, 0)

    @property
    def should_poll(self) -> bool:
        """Tesou is a polled entity."""
        return True

    @property
    def latitude(self) -> float | None:
        """Return latitude value of the device."""
        return self.latlong[0]

    @property
    def longitude(self) -> float | None:
        """Return longitude value of the device."""
        return self.latlong[1]

    @property
    def source_type(self) -> SourceType:
        """Return the source type of the device."""
        return SourceType.GPS

    async def async_update(self) -> None:
        """Get the latest data from the Tesou API."""
        pos = await self.api.get_latest_gps_position(self.user)
        if pos is not None:
            self.latlong = pos.latitude, pos.longitude
        else:
            raise ConnectionError
