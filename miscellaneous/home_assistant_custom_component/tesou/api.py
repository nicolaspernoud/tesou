"""API module for Tesou."""
# import asyncio

import aiohttp


class User:
    """User object with type hints."""

    def __init__(self, user_id: int, name: str, surname: str) -> None:
        """Initialize an User."""
        self.id = user_id
        self.name = name
        self.surname = surname


class Position:
    """Position object with type hints."""

    def __init__(
        self,
        pos_id: int,
        user_id: int,
        latitude: float,
        longitude: float,
        source: str,
        battery_level: int,
        sport_mode: bool,
        time: int,
    ) -> None:
        """Initialize a Position."""
        self.id = pos_id
        self.user_id = user_id
        self.latitude = latitude
        self.longitude = longitude
        self.source = source
        self.battery_level = battery_level
        self.sport_mode = sport_mode
        self.time = time


class TesouApi:
    """Tesou! API client."""

    def __init__(self, host: str, token: str) -> None:
        """Initialize."""
        self.host = host
        self.token = token

    async def get_users(self) -> list[User]:
        """Retrieve users from the host."""
        url = f"{self.host}/api/users"
        headers = {
            "Authorization": f"Bearer {self.token}",
        }

        async with aiohttp.ClientSession() as session, session.get(
            url, headers=headers
        ) as response:
            if response.status == 200:
                # Assuming the response is a JSON array of users
                users_data = await response.json()
                users = [
                    User(user_id=user["id"], name=user["name"], surname=user["surname"])
                    for user in users_data
                ]
                return users
            # Handle other status codes if needed
            return []

    async def get_latest_gps_position(self, user: User) -> Position | None:
        """Retrieve the latest GPS position for a user."""
        url = f"{self.host}/api/positions?user_id={user.id}"
        headers = {
            "Authorization": f"Bearer {self.token}",
        }

        async with aiohttp.ClientSession() as session, session.get(
            url, headers=headers
        ) as response:
            if response.status == 200:
                # Assuming the response is a JSON array of positions
                positions_data = await response.json()
                gps_positions = [
                    Position(
                        pos_id=position["id"],
                        **{k: position[k] for k in position if k != "id"},
                    )
                    for position in positions_data
                    if position["source"] == "GPS"
                ]
                # Find the position with the greatest ID
                latest_gps_position = max(
                    gps_positions, key=lambda position: position.id, default=None
                )
                return latest_gps_position
            # Handle other status codes if needed
            return None


# async def main():
#     """Is an example usage."""
#     host = "https://tesou.****.***"
#     token = "****"

#     tesou_api = TesouApi(host, token)
#     users = await tesou_api.get_users()

#     for user in users:
#         latest_gps_position = await tesou_api.get_latest_gps_position(user)
#         if latest_gps_position:
#             print(f"Latest GPS Position for {user.name} {user.surname}:")  # noqa: T201
#             print(  # noqa: T201
#                 f"ID: {latest_gps_position.id}, Latitude: {latest_gps_position.latitude}, Longitude: {latest_gps_position.longitude}, Source: {latest_gps_position.source}"
#             )
#         else:
#             print(f"No GPS position found for {user.name} {user.surname}")  # noqa: T201


# # Run the example

# asyncio.run(main())
